use crate::Span;

#[derive(Debug)]
pub(crate) struct Name {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Default)]
pub(crate) struct Program {
    pub actors: Vec<Actor>,
    pub scenes: Vec<Scene>,
}

#[derive(Debug)]
pub(crate) struct Actor {
    pub name: Name,
    pub id: String,
}

#[derive(Debug)]
pub(crate) struct Scene {
    pub name: Name,
    pub lines: Vec<Dialogue>,
}

#[derive(Debug)]
pub(crate) struct Dialogue {
    pub actor: Name,
    pub text: String,
}
