
use crate::{graph::ToValue, interpreter::instructions::Block};

pub(crate) fn compile(statements: crate::parser::ast::Statements) -> crate::Result<super::Program>
{
  type Statement = crate::parser::ast::Statement;
  type Instruction = super::instructions::Instruction;
  let program = statements.iter().map(|stmt|
    {
      let mut block = super::instructions::Block::new();
      match stmt {
        Statement::Create(create) => {
          let mut variables = Vec::<Option<String>>::new(); 
          for c in create.patterns.iter()
          {
            match c {
              crate::parser::ast::GraphNodeOrEdge::GraphNode(node) => {
                variables.push(node.variable.to_owned());
                block.push(Instruction::Push { value: crate::graph::Node::default().to_value() });
              }
              crate::parser::ast::GraphNodeOrEdge::GraphEdge(edge) => {
                variables.push(edge.variable.to_owned());
                block.push(Instruction::Push { value: crate::graph::Edge::default().to_value() });
              }
            }
          }
          block.push(Instruction::Create { variables: variables });
          Ok(block)
        },
        _ => { return Err(crate::Error::Unimplemented("compile")) }
      }
    });
  program.collect::<crate::Result<super::Program>>()
}
