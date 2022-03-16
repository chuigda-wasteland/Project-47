use std::collections::HashMap;

use xjbutil::slice_arena::SliceArena;
pub use xjbutil::value::Value;

use crate::diag::DiagContext;
use crate::syntax::decl::{ConcreteDecl, ConcreteImportDecl, ConcreteOpenImportDecl};

pub type SyntaxActionNamespace = &'static [&'static str];
pub use Value as GValue;
use crate::syntax::ConcreteProgram;

pub type TokenLitArena = SliceArena<8192, 1>;

type DataMap = HashMap<(SyntaxActionNamespace, String), Value>;

pub struct DataMapLens<'a> {
    namespace: SyntaxActionNamespace,
    data_map_ref: &'a mut DataMap
}

impl<'a> DataMapLens<'a> {
    fn new(namespace: SyntaxActionNamespace, data_map_ref: &'a mut DataMap) -> Self {
        DataMapLens { namespace, data_map_ref }
    }
}

impl<'a> DataMapLens<'a> {
    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.data_map_ref.insert((self.namespace, key.into()), value);
    }

    pub fn get(&self, key: impl Into<String>) -> Option<&Value> {
        self.data_map_ref.get(&(self.namespace, key.into()))
    }

    pub fn get_mut(&mut self, key: impl Into<String>) -> Option<&mut Value> {
        self.data_map_ref.get_mut(&(self.namespace, key.into()))
    }
}

pub type GlobalSyntaxAction = fn(
    data_map: DataMapLens<'_>,
    diag: &'_ mut DiagContext
);

pub type SyntaxAction<'a> = fn(
    decl: &'_ mut ConcreteDecl<'_>,
    data_map: DataMapLens<'_>,
    arena: &'a TokenLitArena,
    diag: &'_ mut DiagContext
) -> Option<Vec<ConcreteDecl<'a>>>;

pub struct SyntaxActionApplier<'a, 's> {
    data_map: DataMap,
    global_actions: HashMap<(SyntaxActionNamespace, String), Option<SyntaxAction<'s>>>,
    actions: HashMap<(SyntaxActionNamespace, String), Option<SyntaxAction<'s>>>,
    arena: &'s TokenLitArena,
    diag_context: &'a mut DiagContext
}

impl<'a, 's> SyntaxActionApplier<'a, 's> {
    pub fn new(arena: &'s TokenLitArena, diag_context: &'a mut DiagContext) -> Self {
        SyntaxActionApplier {
            data_map: DataMap::new(),
            global_actions: HashMap::new(),
            actions: HashMap::new(),
            arena,
            diag_context
        }
    }

    pub fn apply_actions(&mut self, program: &mut ConcreteProgram<'s>) {
        let mut imported_items: HashMap<String, SyntaxActionNamespace> = HashMap::new();
        for decl in &program.decls {
            match decl {
                ConcreteDecl::ImportDecl(import) =>
                    self.resolve_import(&mut imported_items, import),
                ConcreteDecl::OpenImportDecl(open_import) =>
                    self.resolve_open_import(&mut imported_items, open_import),
                _ => {}
            }
        }
    }

    fn resolve_import(
        &mut self,
        _imported_items: &mut HashMap<String, SyntaxActionNamespace>,
        _import: &ConcreteImportDecl<'s>
    ) {
        todo!()
    }

    fn resolve_open_import(
        &mut self,
        _imported_items: &mut HashMap<String, SyntaxActionNamespace>,
        _open_import: &ConcreteOpenImportDecl<'s>
    ) {
        todo!()
    }
}
