use super::*;
macros::table!(MethodDef);

impl MethodDef {
    pub fn flags(&self) -> MethodFlags {
        MethodFlags(self.reader.u32(self.row, 2))
    }

    pub fn params(&self) -> impl Iterator<Item = Param> + '_ {
        self.reader
            .list(self.row, TableIndex::Param, 5)
            .map(move |row| Param {
                reader: self.reader,
                row,
            })
    }

    pub fn name(&self) -> &'static str {
        self.reader.str(self.row, 3)
    }

    pub fn sig(&self) -> Blob {
        self.reader.blob(self.row, 4)
    }

    pub fn category(&self) -> MethodCategory {
        if self.flags().special() {
            let name = self.name();

            if name.starts_with("get") {
                MethodCategory::Get
            } else if name.starts_with("put") {
                MethodCategory::Set
            } else if name.starts_with("add") {
                MethodCategory::Add
            } else if name.starts_with("remove") {
                MethodCategory::Remove
            } else {
                // A delegate's 'Invoke' method is "special" but lacks a preamble.
                MethodCategory::Normal
            }
        } else {
            MethodCategory::Normal
        }
    }

    pub fn attributes(&self) -> impl Iterator<Item = Attribute> + '_ {
        self.reader
            .equal_range(
                self.row.file_index,
                TableIndex::CustomAttribute,
                0,
                HasAttribute::MethodDef(*self).encode(),
            )
            .map(move |row| Attribute {
                reader: self.reader,
                row,
            })
    }

    pub fn impl_map(&self) -> Option<ImplMap> {
        self.reader
            .equal_range(
                self.row.file_index,
                TableIndex::ImplMap,
                1,
                MemberForwarded::MethodDef(*self).encode(),
            )
            .map(move |row| ImplMap {
                reader: self.reader,
                row,
            })
            .next()
    }
}
