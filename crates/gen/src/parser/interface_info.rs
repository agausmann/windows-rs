use super::*;

pub struct InterfaceInfo {
    pub def: GenericType,
    pub kind: InterfaceKind,
    pub is_base: bool,
    pub version: (u16, u16),
}

impl InterfaceInfo {
    pub fn sort(result: &mut Vec<Self>) {
        result.sort_by(|a, b| {
            if a.is_base != b.is_base {
                return a.is_base.cmp(&b.is_base);
            }

            if a.version != b.version {
                return a.version.cmp(&b.version);
            }

            if a.kind != b.kind {
                return a.kind.cmp(&b.kind);
            }

            a.def.def.full_name().cmp(&b.def.def.full_name())
        });
    }

    pub fn gen_methods(interfaces: &Vec<Self>, gen: Gen) -> TokenStream {
        let mut method_names = BTreeMap::<String, u32>::new();
        // TODO: get rid of all these temporary streams and use iterators. This just
        // ends up creating a bunch of temporary strings under the hood.
        let mut tokens = TokenStream::new();

        for interface in interfaces {
            for (vtable_offset, method) in interface.def.def.methods().enumerate() {
                let name = method.rust_name();
                let overload = method_names.entry(name.clone()).or_insert(0);
                *overload += 1;

                let info = MethodInfo {
                    name,
                    vtable_offset: vtable_offset as u32 + 6,
                    overload: *overload,
                };

                let signature = method.signature(&interface.def.generics);
                tokens.combine(&signature.gen_winrt_method(&info, interface, gen));
            }
        }

        tokens
    }

    pub fn gen_conversion(
        &self,
        from: &TokenStream,
        constraints: &TokenStream,
        gen: Gen,
    ) -> TokenStream {
        if self.def.def.is_exclusive() {
            return TokenStream::new();
        }

        match self.kind {
            InterfaceKind::Default => {
                let into = self.def.gen_name(gen);
                quote! {
                    impl<#constraints> ::std::convert::From<#from> for #into {
                        fn from(value: #from) -> Self {
                            unsafe { ::std::mem::transmute(value) }
                        }
                    }
                    impl<#constraints> ::std::convert::From<&#from> for #into {
                        fn from(value: &#from) -> Self {
                            ::std::convert::From::from(::std::clone::Clone::clone(value))
                        }
                    }
                    impl<'a, #constraints> ::std::convert::Into<::windows::Param<'a, #into>> for #from {
                        fn into(self) -> ::windows::Param<'a, #into> {
                            ::windows::Param::Owned(::std::convert::Into::<#into>::into(self))
                        }
                    }
                    impl<'a, #constraints> ::std::convert::Into<::windows::Param<'a, #into>> for &'a #from {
                        fn into(self) -> ::windows::Param<'a, #into> {
                            ::windows::Param::Owned(::std::convert::Into::<#into>::into(::std::clone::Clone::clone(self)))
                        }
                    }
                }
            }
            InterfaceKind::NonDefault => {
                let into = self.def.gen_name(gen);
                quote! {
                    impl<#constraints> ::std::convert::From<#from> for #into {
                        fn from(value: #from) -> Self {
                            ::std::convert::From::from(&value)
                        }
                    }
                    impl<#constraints> ::std::convert::From<&#from> for #into {
                        fn from(value: &#from) -> Self {
                            ::windows::Interface::cast(value).unwrap()
                        }
                    }
                    impl<'a, #constraints> ::std::convert::Into<::windows::Param<'a, #into>> for #from {
                        fn into(self) -> ::windows::Param<'a, #into> {
                            ::windows::Param::Owned(::std::convert::Into::<#into>::into(self))
                        }
                    }
                    impl<'a, #constraints> ::std::convert::Into<::windows::Param<'a, #into>> for &'a #from {
                        fn into(self) -> ::windows::Param<'a, #into> {
                            ::windows::Param::Owned(::std::convert::Into::<#into>::into(::std::clone::Clone::clone(self)))
                        }
                    }
                }
            }
            _ => TokenStream::new(),
        }
    }
}
