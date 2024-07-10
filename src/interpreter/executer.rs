use std::borrow::BorrowMut;

use crate::interpreter;


pub(crate) fn eval(store: &crate::store::Store, program: super::Program) -> crate::Result<crate::graph::Value>
{
  type Instruction = crate::interpreter::instructions::Instruction;
  let mut tx = store.begin()?;
  let mut stack = Vec::<crate::graph::Value>::new();
  for instruction in program
  {
    match instruction {
      Instruction::Push{value: val} => {
        stack.push(val);
      }
      // { value: crate::graph::Value },
      Instruction::Create{variables: vars} => {
        for v in stack.drain(stack.len() - vars.len()..)
        {
          match v {
            crate::graph::Value::Node(n) => {
              store.add_nodes(tx.borrow_mut(), "default", vec![n].iter())?;
            }
            _ => {
              return Err(crate::Error::Unknown)
            }
          }
        }
      }
      // { variables: Vec<Option<String>> },    
    }
  }
  Ok(crate::graph::Value::Invalid)
}
