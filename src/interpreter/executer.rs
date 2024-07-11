use std::borrow::BorrowMut;

pub(crate) fn eval(store: &crate::store::Store, program: super::Program) -> crate::Result<crate::graph::Value>
{
  let mut input_table = crate::value_table::ValueTable::new();
  input_table.add_row(crate::value_table::Row::new());
  let output_table = crate::value_table::ValueTable::new();
  type Instruction = crate::interpreter::instructions::Instruction;
  let mut tx = store.begin()?;
  let mut stack = Vec::<crate::graph::Value>::new();
  for block in program
  {
    for row in input_table.iter()
    {
      for instruction in block.iter()
      {
        match instruction {
          Instruction::Push{value: val} => {
            stack.push(val.clone());
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
    }
  }
  Ok(crate::graph::Value::Invalid)
}
