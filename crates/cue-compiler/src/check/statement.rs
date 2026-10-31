use super::{Checker, Result};
use crate::{
    Diagnostic,
    ast::{self, StmtKind},
    checked::{self, Access},
    types::Type,
};
use std::collections::{BTreeMap, BTreeSet};

impl Checker {
    pub(super) fn block(&mut self, statements: &[ast::Stmt]) -> Result<Vec<checked::Stmt>> {
        self.scopes.push(BTreeMap::new());
        let result = self.statements(statements);
        self.scopes.pop();
        result
    }

    pub(super) fn statements(&mut self, statements: &[ast::Stmt]) -> Result<Vec<checked::Stmt>> {
        statements
            .iter()
            .map(|statement| self.statement(statement))
            .collect()
    }

    fn statement(&mut self, stmt: &ast::Stmt) -> Result<checked::Stmt> {
        use checked::StmtKind as Checked;
        let kind = match &stmt.kind {
            StmtKind::Dialogue(actor, text) => {
                let id = self.actors.get(&actor.value).cloned().ok_or_else(|| {
                    Diagnostic::new(format!("unknown actor '{}'", actor.value), actor.span)
                })?;
                Checked::Dialogue(id, self.expression(text, Some(&Type::String))?)
            }
            StmtKind::Bind {
                name,
                mutable,
                annotation,
                value,
            } => {
                let ty = annotation
                    .as_ref()
                    .map(|ty| self.resolve_type(ty))
                    .transpose()?;
                let value = self.expression(value, ty.as_ref())?;
                if !value.ty.concrete() {
                    return Err(Diagnostic::new(
                        "cannot infer this value's type; add a type annotation",
                        value.span,
                    ));
                }
                Checked::Bind(self.bind(name, value.ty.clone(), *mutable)?, value)
            }
            StmtKind::Assign(target, value) => {
                let mut path = Vec::new();
                let (local, mut ty) = self.target(target, &mut path)?;
                if path.is_empty()
                    && let ast::ExprKind::Name(name) = &target.kind
                {
                    ty = self.binding(name)?.ty.clone();
                }
                let value = self.expression(value, Some(&ty))?;
                if path.is_empty() {
                    self.refinements.remove(&local);
                }
                Checked::Assign { local, path, value }
            }
            StmtKind::If { condition, yes, no } => {
                let checked_condition = self.expression(condition, Some(&Type::Bool))?;
                let before = self.refinements.clone();
                self.narrow(condition, true);
                let yes = self.block(yes)?;
                let mut after = self.refinements.clone();
                self.refinements = before;
                self.narrow(condition, false);
                let no = self.block(no)?;
                after.retain(|id, ty| self.refinements.get(id) == Some(ty));
                self.refinements = after;
                Checked::If {
                    condition: checked_condition,
                    yes,
                    no,
                }
            }
            StmtKind::For {
                name,
                collection,
                body,
            } => {
                let collection = self.expression(collection, None)?;
                let Type::List(item) = &collection.ty else {
                    return Err(Diagnostic::new("for requires a list", collection.span));
                };
                if !item.concrete() {
                    return Err(Diagnostic::new(
                        "cannot infer the loop item type; annotate the list",
                        collection.span,
                    ));
                }
                let before = self.refinements.clone();
                // Facts established outside a loop must survive every iteration.
                // A later write also invalidates reads at the top of the next one.
                let mut writes = BTreeSet::new();
                assigned_names(body, BTreeSet::from([name.value.clone()]), &mut writes);
                for name in writes {
                    if let Some(binding) =
                        self.scopes.iter().rev().find_map(|scope| scope.get(&name))
                    {
                        self.refinements.remove(&binding.id);
                    }
                }
                self.scopes.push(BTreeMap::new());
                let local = self.bind(name, *item.clone(), false)?;
                let body = self.statements(body)?;
                self.scopes.pop();
                // A loop can run zero times. Only keep facts valid on both paths.
                self.refinements.retain(|id, ty| before.get(id) == Some(ty));
                Checked::For {
                    local,
                    collection,
                    body,
                }
            }
            StmtKind::Match { value, arms } => {
                let value = self.expression(value, None)?;
                let arms = self.match_arms(&value, arms)?;
                Checked::Match { value, arms }
            }
            StmtKind::Block(body) => Checked::Block(self.block(body)?),
        };
        Ok(checked::Stmt {
            kind,
            span: stmt.span,
        })
    }

    fn target(&mut self, expr: &ast::Expr, path: &mut Vec<Access>) -> Result<(usize, Type)> {
        match &expr.kind {
            ast::ExprKind::Name(name) => {
                let binding = self.binding(name)?;
                if !binding.mutable {
                    return Err(Diagnostic::new(
                        format!("cannot assign to immutable local '{}'", name.value),
                        name.span,
                    ));
                }
                Ok((
                    binding.id,
                    self.refinements
                        .get(&binding.id)
                        .unwrap_or(&binding.ty)
                        .clone(),
                ))
            }
            ast::ExprKind::Field(receiver, field) => {
                let (local, ty) = self.target(receiver, path)?;
                let Type::Record(fields) = ty else {
                    return Err(Diagnostic::new(
                        "assignment field requires a non-optional record",
                        receiver.span,
                    ));
                };
                let ty = fields.get(&field.value).cloned().ok_or_else(|| {
                    Diagnostic::new(format!("unknown field '{}'", field.value), field.span)
                })?;
                path.push(Access::Field(field.value.clone()));
                Ok((local, ty))
            }
            ast::ExprKind::Index(receiver, index) => {
                let (local, ty) = self.target(receiver, path)?;
                let Type::List(item) = ty else {
                    return Err(Diagnostic::new(
                        "assignment index requires a non-optional list",
                        receiver.span,
                    ));
                };
                path.push(Access::Index(self.expression(index, Some(&Type::Number))?));
                // Unlike a read, a write must find the existing item at runtime.
                Ok((local, *item))
            }
            _ => Err(Diagnostic::new(
                "assignment target must be a mutable local, field, or index",
                expr.span,
            )),
        }
    }
}

fn assigned_names(
    statements: &[ast::Stmt],
    mut shadowed: BTreeSet<String>,
    writes: &mut BTreeSet<String>,
) {
    for statement in statements {
        match &statement.kind {
            StmtKind::Bind { name, .. } => {
                shadowed.insert(name.value.clone());
            }
            StmtKind::Assign(target, _) => {
                if let ast::ExprKind::Name(name) = &target.kind
                    && !shadowed.contains(&name.value)
                {
                    writes.insert(name.value.clone());
                }
            }
            StmtKind::Block(body) => assigned_names(body, shadowed.clone(), writes),
            StmtKind::If { yes, no, .. } => {
                assigned_names(yes, shadowed.clone(), writes);
                assigned_names(no, shadowed.clone(), writes);
            }
            StmtKind::For { name, body, .. } => {
                let mut inner = shadowed.clone();
                inner.insert(name.value.clone());
                assigned_names(body, inner, writes);
            }
            StmtKind::Match { arms, .. } => {
                for arm in arms {
                    let mut inner = shadowed.clone();
                    if let Some(name) = &arm.binding {
                        inner.insert(name.value.clone());
                    }
                    assigned_names(&arm.body, inner, writes);
                }
            }
            StmtKind::Dialogue(..) => {}
        }
    }
}
