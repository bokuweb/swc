use ast::*;
use swc_atoms::JsWord;
use swc_common::{util::move_map::MoveMap, Fold, FoldWith, Spanned, SyntaxContext, DUMMY_SP};

#[derive(Debug)]
pub(super) enum ScopeOp {
    Rename {
        from: (JsWord, SyntaxContext),
        to: JsWord,
    },
}

pub(super) struct Operator<'a>(pub &'a [ScopeOp]);

impl<'a> Fold<Vec<ModuleItem>> for Operator<'a> {
    fn fold(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let mut stmts = Vec::with_capacity(items.len());

        for item in items {
            let span = item.span();

            macro_rules! export {
                ($orig:expr, $ident:expr) => {
                    stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                        NamedExport {
                            span,
                            specifiers: vec![ExportSpecifier::Named(NamedExportSpecifier {
                                span: DUMMY_SP,
                                orig: $ident,
                                exported: Some($orig),
                            })],
                            src: None,
                        },
                    )));
                };
            }

            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    span,
                    decl:
                        Decl::Class(ClassDecl {
                            ident,
                            class,
                            declare,
                        }),
                })) => {
                    let orig_ident = ident.clone();
                    match self.rename_ident(ident) {
                        Ok(ident) => {
                            stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
                                ident: ident.clone(),
                                class,
                                declare,
                            }))));
                            export!(orig_ident, ident);
                        }
                        Err(ident) => {
                            stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                                span,
                                decl: Decl::Class(ClassDecl {
                                    ident,
                                    class,
                                    declare,
                                }),
                            })))
                        }
                    }
                }
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    span,
                    decl:
                        Decl::Fn(FnDecl {
                            ident,
                            function,
                            declare,
                        }),
                })) => {
                    let orig_ident = ident.clone();
                    match self.rename_ident(ident) {
                        Ok(ident) => {
                            stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
                                ident: ident.clone(),
                                function,
                                declare,
                            }))));
                            export!(orig_ident, ident);
                        }
                        Err(ident) => {
                            stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                                span,
                                decl: Decl::Fn(FnDecl {
                                    ident,
                                    function,
                                    declare,
                                }),
                            })))
                        }
                    }
                }
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Var(var),
                    ..
                })) => {
                    let mut renamed: Vec<ExportSpecifier> = vec![];
                    let decls = var.decls.move_map(|decl| {
                        let name = decl.name.fold_with(&mut VarFolder {
                            orig: self,
                            renamed: &mut renamed,
                        });
                        VarDeclarator { name, ..decl }
                    });
                    if renamed.is_empty() {
                        stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                            span,
                            decl: Decl::Var(VarDecl { decls, ..var }),
                        })));
                        continue;
                    }
                    stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                        decls,
                        ..var
                    }))));
                    stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                        NamedExport {
                            span,
                            specifiers: renamed,
                            src: None,
                        },
                    )));
                }
                _ => stmts.push(item.fold_with(self)),
            }
        }

        stmts
    }
}

struct VarFolder<'a, 'b> {
    orig: &'a mut Operator<'b>,
    renamed: &'a mut Vec<ExportSpecifier>,
}

impl Fold<Ident> for VarFolder<'_, '_> {
    fn fold(&mut self, i: Ident) -> Ident {
        let orig = i.clone();
        match self.orig.rename_ident(i) {
            Ok(i) => {
                self.renamed
                    .push(ExportSpecifier::Named(NamedExportSpecifier {
                        span: i.span,
                        exported: Some(orig),
                        orig: i.clone(),
                    }));
                i
            }
            Err(i) => return i,
        }
    }
}

impl Fold<Expr> for VarFolder<'_, '_> {
    fn fold(&mut self, n: Expr) -> Expr {
        n
    }
}

/// Preserve key of properties.
impl<'a> Fold<KeyValuePatProp> for Operator<'a> {
    fn fold(&mut self, p: KeyValuePatProp) -> KeyValuePatProp {
        KeyValuePatProp {
            value: p.value.fold_with(self),
            ..p
        }
    }
}

impl Fold<ObjectPatProp> for Operator<'_> {
    fn fold(&mut self, p: ObjectPatProp) -> ObjectPatProp {
        let p = p.fold_children(self);

        match p {
            ObjectPatProp::Assign(p) => match self.rename_ident(p.key.clone()) {
                Ok(renamed) => KeyValuePatProp {
                    key: PropName::Ident(p.key),

                    value: box Pat::Ident(renamed),
                }
                .into(),
                Err(_) => p.into(),
            },
            _ => p,
        }
    }
}

/// Preserve key of properties.
impl<'a> Fold<AssignPatProp> for Operator<'a> {
    fn fold(&mut self, p: AssignPatProp) -> AssignPatProp {
        match p.value {
            Some(value) => AssignPatProp {
                value: Some(value.fold_children(self)),
                ..p
            },
            None => p,
        }
    }
}

/// Preserves key
impl<'a> Fold<Prop> for Operator<'a> {
    fn fold(&mut self, prop: Prop) -> Prop {
        match prop {
            Prop::Shorthand(i) => {
                match self.rename_ident(i.clone()) {
                    Ok(renamed) => Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(Ident {
                            // clear mark
                            span: i.span.with_ctxt(SyntaxContext::empty()),
                            ..i
                        }),
                        value: box Expr::Ident(renamed),
                    }),
                    Err(i) => Prop::Shorthand(i),
                }
            }
            _ => prop.fold_children(self),
        }
    }
}

/// Preserve key in object properties.
impl<'a> Fold<KeyValueProp> for Operator<'a> {
    fn fold(&mut self, p: KeyValueProp) -> KeyValueProp {
        KeyValueProp {
            value: p.value.fold_with(self),
            ..p
        }
    }
}

impl<'a> Operator<'a> {
    /// Returns `Ok(renamed_ident)` if ident should be renamed.
    fn rename_ident(&mut self, ident: Ident) -> Result<Ident, Ident> {
        for op in self.0 {
            match *op {
                ScopeOp::Rename { ref from, ref to }
                    if *from.0 == ident.sym && from.1 == ident.span.ctxt() =>
                {
                    return Ok(Ident {
                        // Clear mark
                        span: ident.span.with_ctxt(SyntaxContext::empty()),
                        sym: to.clone(),
                        ..ident
                    });
                }
                _ => {}
            }
        }
        Err(ident)
    }
}

impl Fold<MemberExpr> for Operator<'_> {
    fn fold(&mut self, expr: MemberExpr) -> MemberExpr {
        let span = expr.span.fold_with(self);
        let obj = expr.obj.fold_with(self);

        let prop = if expr.computed {
            expr.prop.fold_with(self)
        } else {
            expr.prop
        };
        MemberExpr {
            span,
            obj,
            prop,
            computed: expr.computed,
        }
    }
}

impl<'a> Fold<Ident> for Operator<'a> {
    fn fold(&mut self, ident: Ident) -> Ident {
        match self.rename_ident(ident) {
            Ok(i) | Err(i) => i,
        }
    }
}

impl<'a> Fold<NamedExportSpecifier> for Operator<'a> {
    fn fold(&mut self, s: NamedExportSpecifier) -> NamedExportSpecifier {
        if s.exported.is_some() {
            return NamedExportSpecifier {
                orig: s.orig.fold_with(self),
                ..s
            };
        }

        let exported = s.orig.clone();

        match self.rename_ident(s.orig) {
            Ok(orig) => NamedExportSpecifier {
                exported: Some(exported),
                orig,
                ..s
            },
            Err(orig) => NamedExportSpecifier { orig, ..s },
        }
    }
}

impl<'a> Fold<ImportSpecific> for Operator<'a> {
    fn fold(&mut self, s: ImportSpecific) -> ImportSpecific {
        if s.imported.is_some() {
            return ImportSpecific {
                local: s.local.fold_with(self),
                ..s
            };
        }

        let imported = s.local.clone();
        let local = self.rename_ident(s.local);

        match local {
            Ok(local) => ImportSpecific {
                imported: Some(imported),
                local,
                ..s
            },
            Err(local) => ImportSpecific { local, ..s },
        }
    }
}
