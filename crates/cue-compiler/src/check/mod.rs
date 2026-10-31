mod expression;
mod statement;

use crate::{Diagnostic, Span, ast, checked, types::Type};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

type Result<T> = std::result::Result<T, Diagnostic>;
type Variants = BTreeMap<String, Option<Type>>;

struct Binding {
    id: usize,
    ty: Type,
    mutable: bool,
}

struct Checker {
    actors: BTreeMap<String, String>,
    enums: BTreeMap<String, Variants>,
    scopes: Vec<BTreeMap<String, Binding>>,
    next_local: usize,
    refinements: BTreeMap<usize, Type>,
}

pub(crate) fn check(
    program: ast::Program,
) -> std::result::Result<checked::Program, Vec<Diagnostic>> {
    let mut checker = Checker {
        actors: BTreeMap::new(),
        enums: BTreeMap::new(),
        scopes: Vec::new(),
        next_local: 0,
        refinements: BTreeMap::new(),
    };
    let mut errors = Vec::new();
    for actor in program.actors {
        match checker.actors.entry(actor.name.value) {
            Entry::Occupied(entry) => errors.push(Diagnostic::new(
                format!("actor '{}' is already defined", entry.key()),
                actor.name.span,
            )),
            Entry::Vacant(entry) => {
                entry.insert(actor.id);
            }
        }
    }
    for enumeration in &program.enums {
        let name = &enumeration.name;
        if matches!(name.value.as_str(), "number" | "bool" | "string")
            || checker
                .enums
                .insert(name.value.clone(), BTreeMap::new())
                .is_some()
        {
            errors.push(Diagnostic::new(
                format!("type '{}' is already defined", name.value),
                name.span,
            ));
        }
    }
    for enumeration in &program.enums {
        let mut variants = BTreeMap::new();
        if enumeration.variants.is_empty() {
            errors.push(Diagnostic::new(
                "an enum needs at least one variant",
                enumeration.name.span,
            ));
        }
        for (name, payload) in &enumeration.variants {
            let ty = match payload
                .as_ref()
                .map(|ty| checker.resolve_type(ty))
                .transpose()
            {
                Ok(ty) => ty,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
            if variants.insert(name.value.clone(), ty).is_some() {
                errors.push(Diagnostic::new(
                    format!("variant '{}' is already defined", name.value),
                    name.span,
                ));
            }
        }
        checker
            .enums
            .insert(enumeration.name.value.clone(), variants);
    }
    let mut names = BTreeSet::new();
    let mut scenes = Vec::new();
    for scene in program.scenes {
        if !names.insert(scene.name.value.clone()) {
            errors.push(Diagnostic::new(
                format!("scene '{}' is already defined", scene.name.value),
                scene.name.span,
            ));
        }
        checker.scopes.clear();
        checker.next_local = 0;
        checker.refinements.clear();
        match checker.block(&scene.body) {
            Ok(body) => scenes.push(checked::Scene {
                name: scene.name.value,
                locals: checker.next_local,
                body,
            }),
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        Ok(checked::Program { scenes })
    } else {
        Err(errors)
    }
}

impl Checker {
    fn resolve_type(&self, expr: &ast::TypeExpr) -> Result<Type> {
        Ok(match &expr.kind {
            ast::TypeKind::Name(name) => match name.as_str() {
                "number" => Type::Number,
                "bool" => Type::Bool,
                "string" => Type::String,
                _ if self.enums.contains_key(name) => Type::Enum(name.clone()),
                _ => return Err(Diagnostic::new(format!("unknown type '{name}'"), expr.span)),
            },
            ast::TypeKind::List(ty) => Type::List(Box::new(self.resolve_type(ty)?)),
            ast::TypeKind::Optional(ty) => self.resolve_type(ty)?.optional(),
            ast::TypeKind::Record(fields) => {
                let mut types = BTreeMap::new();
                for (name, ty) in fields {
                    if types
                        .insert(name.value.clone(), self.resolve_type(ty)?)
                        .is_some()
                    {
                        return Err(Diagnostic::new(
                            format!("duplicate field '{}'", name.value),
                            name.span,
                        ));
                    }
                }
                Type::Record(types)
            }
        })
    }

    fn binding(&self, name: &ast::Name) -> Result<&Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&name.value))
            .ok_or_else(|| Diagnostic::new(format!("unknown name '{}'", name.value), name.span))
    }

    fn bind(&mut self, name: &ast::Name, ty: Type, mutable: bool) -> Result<usize> {
        let scope = self.scopes.last_mut().expect("bindings belong to a block");
        if scope.contains_key(&name.value) {
            return Err(Diagnostic::new(
                format!("local '{}' is already defined in this block", name.value),
                name.span,
            ));
        }
        let id = self.next_local;
        self.next_local += 1;
        scope.insert(name.value.clone(), Binding { id, ty, mutable });
        Ok(id)
    }

    fn require(&self, expected: &Type, actual: &Type, span: Span) -> Result<()> {
        if expected.accepts(actual) {
            Ok(())
        } else {
            Err(Diagnostic::new(
                format!("expected {expected}, found {actual}"),
                span,
            ))
        }
    }
}
