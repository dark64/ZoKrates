use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;
use zokrates_core::ir::Prog;
use zokrates_core::typed_absy::Type;
use zokrates_field::field::Field;

pub struct Input {
    name: String,
    public: bool,
    internal_type: Type,
}

#[derive(serde_derive::Serialize)]
pub struct Abi {
    inputs: Vec<Input>,
}

impl Abi {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

impl fmt::Display for Abi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json().as_str())
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Input", 3)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("public", &self.public)?;
        s.serialize_field("type", &self.internal_type.get_name())?;
        s.end()
    }
}

pub trait NameDef {
    fn get_name(&self) -> String;
}

impl NameDef for Type {
    fn get_name(&self) -> String {
        match self {
            Type::FieldElement => String::from("field"),
            Type::Boolean => String::from("bool"),
            Type::Array(box ty, size) => format!("{}[{}]", ty.get_name(), size),
            Type::Struct(_) => String::from("struct"),
        }
    }
}

pub trait Generator {
    fn generate_abi(&self) -> Abi;
}

impl<T: Field> Generator for Prog<T> {
    fn generate_abi(&self) -> Abi {
        let inputs: Vec<_> = self
            .signature
            .inputs
            .iter()
            .map(|t| Input {
                name: String::from("hello"),
                public: true,
                internal_type: t.clone(),
            })
            .collect();

        Abi { inputs }
    }
}

#[cfg(test)]
mod tests {
    extern crate zokrates_core;
    extern crate zokrates_field;

    use crate::abi_gen::{Abi, Generator};
    use zokrates_core::flat_absy::FlatVariable;
    use zokrates_core::ir::{Function, Prog, Statement};
    use zokrates_core::typed_absy::{Signature, Type};
    use zokrates_field::field::FieldPrime;

    #[test]
    fn generate_abi() {
        let program: Prog<FieldPrime> = Prog {
            main: Function {
                id: String::from("main"),
                arguments: vec![FlatVariable::new(0)],
                returns: vec![FlatVariable::public(0)],
                statements: vec![Statement::Constraint(
                    FlatVariable::new(0).into(),
                    FlatVariable::public(0).into(),
                )],
            },
            private: vec![false],
            signature: Signature::new()
                .inputs(vec![Type::FieldElement])
                .outputs(vec![Type::FieldElement]),
        };

        let abi: Abi = program.generate_abi();
        println!("{}", abi);
    }
}
