use std::fmt;
use zokrates_core::ir::Prog;
use zokrates_core::typed_absy::Type;
use zokrates_field::field::Field;

#[derive(serde_derive::Serialize)]
struct InnerComponent {
    name: String,
    #[serde(rename = "type")]
    internal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<InnerComponent>>,
}

#[derive(serde_derive::Serialize)]
pub struct InputComponent {
    name: String,
    public: bool,
    #[serde(rename = "type")]
    internal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<InnerComponent>>,
}

#[derive(serde_derive::Serialize)]
pub struct Abi {
    inputs: Vec<InputComponent>
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

trait AbiType {
    fn get_name(&self) -> String;
    fn get_inner_components(&self) -> Option<Vec<InnerComponent>>;
}

impl AbiType for Type {
    fn get_name(&self) -> String {
        match self {
            Type::FieldElement => String::from("field"),
            Type::Boolean => String::from("bool"),
            Type::Array(box ty, size) => format!("{}[{}]", ty.get_name(), size),
            Type::Struct(_) => String::from("struct"),
        }
    }

    fn get_inner_components(&self) -> Option<Vec<InnerComponent>> {
        match self {
            Type::Struct(vec) => {
                let mut components: Vec<InnerComponent> = Vec::new();
                for element in vec {
                    let ty = &element.1;
                    components.push(
                        InnerComponent {
                            name: String::default(),
                            internal_type: ty.get_name(),
                            components: ty.get_inner_components(),
                        }
                    )
                }
                Some(components)
            }
            _ => Option::None
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
            .map(|t| InputComponent {
                name: String::default(),
                public: true,
                internal_type: t.get_name(),
                components: t.get_inner_components(),
            })
            .collect();

        Abi { inputs }
    }
}

#[cfg(test)]
mod tests {
    extern crate zokrates_core;
    extern crate zokrates_field;

    use zokrates_core::flat_absy::FlatVariable;
    use zokrates_core::ir::{Function, Prog, Statement};
    use zokrates_core::typed_absy::{Signature, Type};
    use zokrates_field::field::FieldPrime;

    use crate::abi_gen::{Abi, Generator};

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
