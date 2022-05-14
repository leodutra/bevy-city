use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Triangle {
    vertex1: u16,
    vertex2: u16,
    vertex3: u16,
    material_id: u16,
}
impl Triangle {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (vertex2, vertex1, material_id, vertex3)) =
            tuple((nc::le_u16, nc::le_u16, nc::le_u16, nc::le_u16))(input)?;
        Ok((
            input,
            Triangle {
                vertex1,
                vertex2,
                vertex3,
                material_id,
            },
        ))
    }
}
