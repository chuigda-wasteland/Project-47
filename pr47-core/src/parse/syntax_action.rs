use std::collections::HashMap;
use std::hint::unreachable_unchecked;

use xjbutil::slice_arena::SliceArena;
pub use xjbutil::value::Value;

use crate::diag::{DiagContext, diag_data};
use crate::syntax::ConcreteProgram;
use crate::syntax::attr::AttrItem;
use crate::syntax::decl::{
    ConcreteDecl,
    ConcreteImportDecl,
    ConcreteOpenImportDecl,
    OpenImportUsingItem
};
use crate::syntax::id::Identifier;

pub use Value as GValue;

pub type TokenLitArena = SliceArena<8192, 1>;

type DataMap = HashMap<(&'static str, String), Value>;

pub struct DataMapLens<'a> {
    namespace: &'static str,
    data_map_ref: &'a mut DataMap
}

impl<'a> DataMapLens<'a> {
    pub(crate) fn new(namespace: &'static str, data_map_ref: &'a mut DataMap) -> Self {
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
    attr: &'_ AttrItem<'_>,
    data_map: DataMapLens<'_>,
    diag: &'_ mut DiagContext
);

pub type SyntaxAction<'a> = fn(
    decl: &'_ mut ConcreteDecl<'a>,
    data_map: DataMapLens<'_>,
    arena: &'a TokenLitArena,
    diag: &'_ mut DiagContext
) -> Option<Vec<ConcreteDecl<'a>>>;

pub struct SyntaxActionApplier<'s, 'd> {
    data_map: DataMap,
    global_actions: HashMap<(&'static str, &'static str), Option<SyntaxAction<'s>>>,
    actions: HashMap<(&'static str, &'static str), Option<SyntaxAction<'s>>>,
    arena: &'s TokenLitArena,
    diag: &'d mut DiagContext
}

impl<'s, 'd> SyntaxActionApplier<'s, 'd> {
    pub fn new(arena: &'s TokenLitArena, diag: &'d mut DiagContext) -> Self {
        SyntaxActionApplier {
            data_map: DataMap::new(),
            global_actions: HashMap::new(),
            actions: HashMap::new(),
            arena,
            diag
        }
    }

    pub fn apply_actions(&mut self, program: &'s mut ConcreteProgram<'s>) {
        let mut imported_items: HashMap<&'s str, (&'s str, &'s str)> = HashMap::new();
        for decl in program.decls.iter_mut() {
            match decl {
                ConcreteDecl::ImportDecl(import) =>
                    self.resolve_import(&mut imported_items, import),
                ConcreteDecl::OpenImportDecl(open_import) =>
                    self.resolve_open_import(&mut imported_items, open_import),
                _ => {}
            }
        }
    }
}

impl<'s, 'd> SyntaxActionApplier<'s, 'd> {
    fn resolve_import(
        &mut self,
        imported_items: &mut HashMap<&'s str, (&'s str, &'s str)>,
        import: &'s mut ConcreteImportDecl<'s>
    ) {
        if let Identifier::Qual(tokens) = &import.import_path {
            if tokens.len() == 2 {
                let namespace: &str = tokens[0].get_str_value();
                let name: &str = tokens[1].get_str_value();

                if !(self.global_actions.contains_key(&(namespace, name))
                     || self.actions.contains_key(&(namespace, name))) {
                    return;
                }

                import.is_syntax_action = true;
                if imported_items.contains_key(name) {
                    self.diag.diag(tokens[0].range.left(),
                                   diag_data::err_duplicate_syntax_action_name_0)
                        .add_arg(name)
                        .emit();
                } else {
                    imported_items.insert(name, (namespace, name));
                }
            }
        }
    }

    // TODO remove this once this function is completed
    #[allow(clippy::collapsible_match)]
    fn resolve_open_import(
        &mut self,
        imported_items: &mut HashMap<&'s str, (&'s str, &'s str)>,
        open_import: &mut ConcreteOpenImportDecl<'s>
    ) {
        if let Identifier::Unqual(namespace_token) = &mut open_import.import_path {
            for used_item /*: OpenImportUsingItem*/ in open_import.use_item_list.iter_mut() {
                if let OpenImportUsingItem::UsingIdent {
                    ident, as_ident, is_syntax_action
                } = used_item {
                    if let Identifier::Unqual(name_token) = ident {
                        let as_name: &'s str = if let Some(as_ident) = as_ident {
                            if let Identifier::Unqual(as_name_token) = as_ident {
                                as_name_token.get_str_value()
                            } else {
                                unsafe { unreachable_unchecked() }
                            }
                        } else {
                            name_token.get_str_value()
                        };

                        let namespace: &'s str = namespace_token.get_str_value();
                        let name = name_token.get_str_value();

                        if !(self.global_actions.contains_key(&(namespace, name))
                             || self.actions.contains_key(&(namespace, name))) {
                            continue;
                        }

                        *is_syntax_action = true;
                        if imported_items.contains_key(as_name) {
                            self.diag.diag(name_token.range.left(),
                                           diag_data::err_duplicate_syntax_action_name_0)
                                .add_arg(namespace_token.get_str_value())
                                .add_arg(as_name)
                                .emit();
                        } else {
                            imported_items.insert(as_name, (namespace, name));
                        }
                    }
                }
            }
        }
    }
}
