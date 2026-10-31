use super::{Checker, Result};
use crate::{
    Diagnostic,
    ast::{self, Binary, Unary},
    checked::{Expr, ExprKind},
    types::Type,
};
use std::collections::{BTreeMap, BTreeSet};

impl Checker {
    pub(super) fn expression(&mut self, expr: &ast::Expr, expected: Option<&Type>) -> Result<Expr> {
        let mut value = self.infer(expr, expected)?;
        if let Some(expected) = expected {
            self.require(expected, &value.ty, expr.span)?;
            value.ty = expected.clone();
        }
        Ok(value)
    }

    fn infer(&mut self, expr: &ast::Expr, expected: Option<&Type>) -> Result<Expr> {
        let context = match expected {
            Some(Type::Optional(ty)) => Some(ty.as_ref()),
            ty => ty,
        };
        let (kind, ty) = match &expr.kind {
            ast::ExprKind::Number(value) => (ExprKind::Number(*value), Type::Number),
            ast::ExprKind::Bool(value) => (ExprKind::Bool(*value), Type::Bool),
            ast::ExprKind::Text(value) => (ExprKind::Text(value.clone()), Type::String),
            ast::ExprKind::None => (ExprKind::None, Type::None),
            ast::ExprKind::Name(name) => {
                let binding = self.binding(name)?;
                (
                    ExprKind::Local(binding.id),
                    self.refinements
                        .get(&binding.id)
                        .unwrap_or(&binding.ty)
                        .clone(),
                )
            }
            ast::ExprKind::List(items) => {
                let item_context = match context {
                    Some(Type::List(ty)) => Some(ty.as_ref()),
                    _ => None,
                };
                let mut ty = item_context.cloned().unwrap_or(Type::Unknown);
                let mut values = Vec::new();
                for item in items {
                    let value = self.expression(item, item_context)?;
                    ty = ty.join(&value.ty).ok_or_else(|| {
                        Diagnostic::new(
                            format!("list items must have one type; found {ty} and {}", value.ty),
                            item.span,
                        )
                    })?;
                    values.push(value);
                }
                (ExprKind::List(values), Type::List(Box::new(ty)))
            }
            ast::ExprKind::Record(fields) => {
                let field_context = match context {
                    Some(Type::Record(fields)) => Some(fields),
                    _ => None,
                };
                let mut types = BTreeMap::new();
                let mut values = Vec::new();
                for (name, value) in fields {
                    if types.contains_key(&name.value) {
                        return Err(Diagnostic::new(
                            format!("duplicate field '{}'", name.value),
                            name.span,
                        ));
                    }
                    if field_context.is_some_and(|fields| !fields.contains_key(&name.value)) {
                        return Err(Diagnostic::new(
                            format!("unknown field '{}'", name.value),
                            name.span,
                        ));
                    }
                    let value = self.expression(
                        value,
                        field_context.and_then(|fields| fields.get(&name.value)),
                    )?;
                    types.insert(name.value.clone(), value.ty.clone());
                    values.push((name.value.clone(), value));
                }
                if let Some(fields) = field_context {
                    for (name, ty) in fields {
                        if !types.contains_key(name) {
                            if !matches!(ty, Type::Optional(_)) {
                                return Err(Diagnostic::new(
                                    format!("missing field '{name}'"),
                                    expr.span,
                                ));
                            }
                            values.push((
                                name.clone(),
                                Expr {
                                    kind: ExprKind::None,
                                    ty: ty.clone(),
                                    span: expr.span,
                                },
                            ));
                        }
                    }
                    types = fields.clone();
                }
                (ExprKind::Record(values), Type::Record(types))
            }
            ast::ExprKind::Unary(operator, operand) => {
                let ty = match operator {
                    Unary::Negate => Type::Number,
                    Unary::Not => Type::Bool,
                };
                let value = self.expression(operand, Some(&ty))?;
                (ExprKind::Unary(*operator, Box::new(value)), ty)
            }
            ast::ExprKind::Binary(operator, left, right) => {
                return self.binary(*operator, left, right, expr);
            }
            ast::ExprKind::Field(receiver, field) => {
                if let Some(value) = self.enum_value(receiver, field, None, expr)? {
                    return Ok(value);
                }
                let receiver = self.expression(receiver, None)?;
                let Type::Record(fields) = &receiver.ty else {
                    return Err(Diagnostic::new(
                        format!("field access requires a record, found {}", receiver.ty),
                        receiver.span,
                    ));
                };
                let ty = fields.get(&field.value).cloned().ok_or_else(|| {
                    Diagnostic::new(format!("unknown field '{}'", field.value), field.span)
                })?;
                (ExprKind::Field(Box::new(receiver), field.value.clone()), ty)
            }
            ast::ExprKind::Index(receiver, index) => {
                let receiver = self.expression(receiver, None)?;
                let Type::List(item) = &receiver.ty else {
                    return Err(Diagnostic::new(
                        format!("indexing requires a list, found {}", receiver.ty),
                        receiver.span,
                    ));
                };
                let ty = item.as_ref().clone().optional();
                let index = self.expression(index, Some(&Type::Number))?;
                (ExprKind::Index(Box::new(receiver), Box::new(index)), ty)
            }
            ast::ExprKind::Call(callee, args) => {
                if let ast::ExprKind::Field(receiver, field) = &callee.kind
                    && let Some(value) = self.enum_value(receiver, field, Some(args), expr)?
                {
                    return Ok(value);
                }
                let ast::ExprKind::Name(name) = &callee.kind else {
                    return Err(Diagnostic::new(
                        "expected an enum constructor or built-in function",
                        callee.span,
                    ));
                };
                if self.binding(name).is_ok() || !matches!(name.value.as_str(), "len" | "string") {
                    return Err(Diagnostic::new(
                        format!("unknown function '{}'", name.value),
                        name.span,
                    ));
                }
                if args.len() != 1 {
                    return Err(Diagnostic::new("expected one argument", expr.span));
                }
                let value = self.expression(&args[0], None)?;
                if name.value == "len" {
                    if !matches!(value.ty, Type::List(_)) {
                        return Err(Diagnostic::new("len requires a list", value.span));
                    }
                    (ExprKind::Length(Box::new(value)), Type::Number)
                } else {
                    if !matches!(value.ty, Type::Number | Type::Bool | Type::String) {
                        return Err(Diagnostic::new(
                            "string requires a number, bool, or string",
                            value.span,
                        ));
                    }
                    (ExprKind::String(Box::new(value)), Type::String)
                }
            }
        };
        Ok(Expr {
            kind,
            ty,
            span: expr.span,
        })
    }

    fn enum_value(
        &mut self,
        receiver: &ast::Expr,
        variant: &ast::Name,
        args: Option<&[ast::Expr]>,
        expr: &ast::Expr,
    ) -> Result<Option<Expr>> {
        let ast::ExprKind::Name(name) = &receiver.kind else {
            return Ok(None);
        };
        if self.binding(name).is_ok() {
            return Ok(None);
        }
        let Some(variants) = self.enums.get(&name.value) else {
            return Ok(None);
        };
        let payload_type = variants
            .get(&variant.value)
            .ok_or_else(|| {
                Diagnostic::new(
                    format!("unknown variant '{}.{}'", name.value, variant.value),
                    variant.span,
                )
            })?
            .clone();
        let payload = match (payload_type, args) {
            (None, None) => None,
            (Some(ty), Some(args)) if args.len() == 1 => {
                Some(Box::new(self.expression(&args[0], Some(&ty))?))
            }
            (None, _) => {
                return Err(Diagnostic::new(
                    "this variant has no payload; omit the parentheses",
                    expr.span,
                ));
            }
            (Some(_), _) => {
                return Err(Diagnostic::new(
                    "this variant requires one payload",
                    expr.span,
                ));
            }
        };
        Ok(Some(Expr {
            kind: ExprKind::Enum {
                enumeration: name.value.clone(),
                variant: variant.value.clone(),
                payload,
            },
            ty: Type::Enum(name.value.clone()),
            span: expr.span,
        }))
    }

    fn binary(
        &mut self,
        operator: Binary,
        left: &ast::Expr,
        right: &ast::Expr,
        expr: &ast::Expr,
    ) -> Result<Expr> {
        use Binary::*;
        let required = match operator {
            Add | Subtract | Multiply | Divide | Remainder | Power | Less | LessEqual | Greater
            | GreaterEqual => Some(Type::Number),
            And | Or => Some(Type::Bool),
            Concat => Some(Type::String),
            _ => None,
        };
        let lhs = self.expression(left, required.as_ref())?;
        let saved = self.refinements.clone();
        if operator == And {
            self.narrow(left, true);
        }
        if operator == Or {
            self.narrow(left, false);
        }
        let right_context = if operator == Coalesce {
            match &lhs.ty {
                Type::Optional(inner) => Some(inner.as_ref()),
                _ => None,
            }
        } else {
            required.as_ref()
        };
        let rhs = self.expression(right, right_context);
        self.refinements = saved;
        let rhs = rhs?;
        let ty = match operator {
            Equal | NotEqual => {
                let declared = |expr: &ast::Expr| match &expr.kind {
                    ast::ExprKind::Name(name) => self.binding(name).ok().map(|b| &b.ty),
                    _ => None,
                };
                let lhs_type = declared(left).unwrap_or(&lhs.ty);
                let rhs_type = declared(right).unwrap_or(&rhs.ty);
                if !lhs.ty.comparable(&rhs.ty) && !lhs_type.comparable(rhs_type) {
                    return Err(Diagnostic::new(
                        format!("cannot compare {} with {}", lhs.ty, rhs.ty),
                        expr.span,
                    ));
                }
                Type::Bool
            }
            Coalesce => {
                let Type::Optional(inner) = &lhs.ty else {
                    return Err(Diagnostic::new(
                        format!("'??' requires an optional value, found {}", lhs.ty),
                        left.span,
                    ));
                };
                self.require(inner, &rhs.ty, right.span)?;
                *inner.clone()
            }
            Less | LessEqual | Greater | GreaterEqual | And | Or => Type::Bool,
            Concat => Type::String,
            _ => Type::Number,
        };
        Ok(Expr {
            kind: ExprKind::Binary(operator, Box::new(lhs), Box::new(rhs)),
            ty,
            span: expr.span,
        })
    }

    pub(super) fn narrow(&mut self, expr: &ast::Expr, truth: bool) {
        match &expr.kind {
            ast::ExprKind::Binary(Binary::And, a, b) if truth => {
                self.narrow(a, true);
                self.narrow(b, true);
            }
            ast::ExprKind::Binary(Binary::Or, a, b) if !truth => {
                self.narrow(a, false);
                self.narrow(b, false);
            }
            ast::ExprKind::Unary(Unary::Not, value) => self.narrow(value, !truth),
            ast::ExprKind::Binary(operator @ (Binary::Equal | Binary::NotEqual), a, b) => {
                if (*operator == Binary::NotEqual) != truth {
                    return;
                }
                let name = match (&a.kind, &b.kind) {
                    (ast::ExprKind::Name(name), ast::ExprKind::None)
                    | (ast::ExprKind::None, ast::ExprKind::Name(name)) => name,
                    _ => return,
                };
                if let Ok(binding) = self.binding(name)
                    && let Type::Optional(inner) = &binding.ty
                {
                    self.refinements.insert(binding.id, *inner.clone());
                }
            }
            _ => {}
        }
    }

    pub(super) fn match_arms(
        &mut self,
        value: &Expr,
        arms: &[ast::Arm],
    ) -> Result<Vec<crate::checked::Arm>> {
        let Type::Enum(name) = &value.ty else {
            return Err(Diagnostic::new("match requires an enum value", value.span));
        };
        let variants = self.enums[name].clone();
        let mut seen = BTreeSet::new();
        let mut result = Vec::new();
        let before = self.refinements.clone();
        let mut after = before.clone();
        for arm in arms {
            self.refinements = before.clone();
            if arm.enumeration.value != *name {
                return Err(Diagnostic::new(
                    format!("expected a variant of {name}"),
                    arm.enumeration.span,
                ));
            }
            let payload = variants.get(&arm.variant.value).ok_or_else(|| {
                Diagnostic::new(
                    format!("unknown variant '{}.{}'", name, arm.variant.value),
                    arm.variant.span,
                )
            })?;
            if !seen.insert(arm.variant.value.clone()) {
                return Err(Diagnostic::new("duplicate match arm", arm.variant.span));
            }
            self.scopes.push(BTreeMap::new());
            let binding = match (&arm.binding, payload) {
                (Some(binding), Some(ty)) => Some(self.bind(binding, ty.clone(), false)?),
                (None, None) => None,
                _ => {
                    return Err(Diagnostic::new(
                        "match payload binding must agree with the variant",
                        arm.variant.span,
                    ));
                }
            };
            let body = self.statements(&arm.body)?;
            self.scopes.pop();
            after.retain(|id, ty| self.refinements.get(id) == Some(ty));
            result.push(crate::checked::Arm {
                variant: arm.variant.value.clone(),
                binding,
                body,
            });
        }
        if seen.len() != variants.len() {
            let missing = variants
                .keys()
                .filter(|v| !seen.contains(*v))
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Diagnostic::new(
                format!("match is missing variants: {missing}"),
                value.span,
            ));
        }
        self.refinements = after;
        Ok(result)
    }
}
