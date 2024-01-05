pub trait Visitor<T, E> where T: Walkable {
    fn visit(&mut self, node: &T) -> Result<(), E>;
}

pub trait MutVisitor<T, E> where T: MutWalkable {
    fn visit_mut(&mut self, node: &mut T) -> Result<(), E>;
}

pub trait Walkable: Sized {
    fn walk<E>(&self, visitor: &mut impl Visitor<Self, E>) -> Result<(), E>;
}

pub trait MutWalkable: Sized {
    fn walk_mut<E>(&self, visitor: &mut impl MutVisitor<Self, E>) -> Result<(), E>;
}

