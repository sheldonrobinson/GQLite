
use crate::graph::ToValue;

pub(crate) fn compile(statements: crate::parser::ast::Statements) -> crate::Result<super::Program>
{
  // let mut program = Vec::<super::instructions::Instruction>::new();
  type Statement = crate::parser::ast::Statement;
  type Instruction = super::instructions::Instruction;
  let mut program = super::Program::new();
  for stmt in statements
  {
    match stmt {
      Statement::Create(create) => {
        let mut variables = Vec::<Option<String>>::new(); 
        for c in create.patterns
        {
          match c {
            crate::parser::ast::GraphNodeOrEdge::GraphNode(node) => {
              variables.push(node.variable);
              program.push(Instruction::Push { value: crate::graph::Node::default().to_value() })
            }
            crate::parser::ast::GraphNodeOrEdge::GraphEdge(edge) => {
              variables.push(edge.variable);
              program.push(Instruction::Push { value: crate::graph::Edge::default().to_value() })
            }
          }
        }
        program.push(Instruction::Create { variables: variables });
      },
      _ => { return Err(crate::Error::Unimplemented("compile")) }
    }
  }
  Ok(program)
}
