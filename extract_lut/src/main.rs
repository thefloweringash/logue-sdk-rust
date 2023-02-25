use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::ops::Deref;

use llvm_ir::constant::{Float, GetElementPtr};
use llvm_ir::name::Name;
use llvm_ir::types::FPType;
use llvm_ir::{module::Linkage, Constant, Module};
use llvm_ir::{ConstantRef, Type};

fn name_to_string(name: &llvm_ir::Name) -> String {
    let name = match name.to_owned() {
        Name::Name(s) => s,
        _ => todo!("non-string symbol"),
    };
    *name
}

trait LUTConstantType {
    const TYPE_NAME: &'static str;
    fn get_value(cr: &ConstantRef) -> Self;
}

impl LUTConstantType for f32 {
    const TYPE_NAME: &'static str = "f32";
    fn get_value(cr: &ConstantRef) -> Self {
        match cr.deref() {
            Constant::Float(Float::Single(x)) => *x,
            _ => {
                panic!("Non-f32 member of LUT: {:?}", cr);
            }
        }
    }
}

impl LUTConstantType for u8 {
    const TYPE_NAME: &'static str = "u8";
    fn get_value(cr: &ConstantRef) -> Self {
        match cr.deref() {
            Constant::Int { bits: 8, value } => u8::try_from(*value).unwrap(),
            _ => {
                panic!("Non-u8 member of LUT: {:?}", cr);
            }
        }
    }
}

fn dump_lut<T: LUTConstantType + Display>(name: &str, public: bool, elements: &Vec<ConstantRef>) {
    println!("#[allow(non_upper_case_globals)]");
    if public {
        println!("#[no_mangle]");
    }
    println!(
        "{priv}static {name}: [{type}; {len}] = [",
        priv = if public { "pub " } else { "" },
        name = name,
        type = T::TYPE_NAME,
        len = elements.len(),
    );
    for el in elements {
        println!("  {x}{type},", x = T::get_value(&el), type = T::TYPE_NAME);
    }
    println!("];");
}

fn dump_lut_spine<T: LUTConstantType>(
    name: &str,
    public: bool,
    elements: &Vec<ConstantRef>,
    sizes: &HashMap<String, usize>,
) {
    fn element_name(el: &ConstantRef) -> String {
        let constant = el.deref();
        match constant {
            Constant::GetElementPtr(GetElementPtr { address, .. }) => match address.deref() {
                Constant::GlobalReference { name, .. } => name_to_string(name),
                _ => panic!("Unexpected entry in spine: {:?}", constant),
            },
            _ => {
                panic!("Non-float member of LUT spine: {:?}", constant);
            }
        }
    }

    let first_element_size = {
        if let Some(first) = elements.first() {
            let name = element_name(first);
            sizes
                .get(&name)
                .unwrap_or_else(|| panic!("Size of lut {name}"))
        } else {
            todo!();
        }
    };

    if !elements.iter().all(|x| {
        sizes
            .get(&element_name(x))
            .unwrap_or_else(|| panic!("Size of {x}"))
            == first_element_size
    }) {
        panic!("irregular array")
    }

    println!("#[allow(non_upper_case_globals)]");
    if public {
        println!("#[no_mangle]");
    }
    println!(
        "{priv}static {name}: [&'static [{type}; {element_size}]; {len}] = [",
        priv = if public { "pub " } else { "" },
        name = name,
        type = T::TYPE_NAME,
        len = elements.len(),
        element_size = first_element_size,
    );
    for el in elements {
        let name = element_name(el);
        println!("  &{},", name);
    }
    println!("];");
}

fn process(path: &str) {
    let module = Module::from_bc_path(path).expect("load module");
    let mut lut_sizes = HashMap::<String, usize>::new();
    for var in module.global_vars {
        if var.linkage == Linkage::Appending {
            // Probably not a LUT
            continue;
        }

        let name = name_to_string(&var.name);

        let public = match var.linkage {
            Linkage::External => true,
            Linkage::Internal => false,
            _ => todo!("var.linkage: {:?}", var.linkage),
        };

        if var.initializer.is_none() {
            // No data, so not a LUT
            continue;
        }

        if let Constant::Array {
            elements,
            element_type,
        } = var.initializer.unwrap().deref()
        {
            match element_type.deref() {
                Type::FPType(FPType::Single) => {
                    lut_sizes.insert(name.clone(), elements.len());
                    dump_lut::<f32>(&name, public, elements);
                }
                Type::IntegerType { bits: 8 } => {
                    lut_sizes.insert(name.clone(), elements.len());
                    dump_lut::<u8>(&name, public, elements);
                }
                Type::PointerType { pointee_type, .. } => match pointee_type.deref() {
                    Type::FPType(FPType::Single) => {
                        dump_lut_spine::<f32>(&name, public, elements, &lut_sizes);
                    }
                    _ => todo!("Non f32 element type: {:?}", element_type),
                },
                _ => todo!(
                    "Non-pointer element type: var: {}, {:?}",
                    var.name,
                    element_type
                ),
            }
        }
    }
}

fn main() {
    for arg in env::args().skip(1) {
        process(&arg)
    }
}
