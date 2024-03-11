use subenum::subenum;

#[subenum(FinalPiece, InterPiece, RawMaterial)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "piece_kind")]
pub enum PieceKind {
    #[subenum(RawMaterial)]
    P1,
    #[subenum(RawMaterial)]
    P2,
    #[subenum(InterPiece)]
    P3,
    #[subenum(InterPiece)]
    P4,
    #[subenum(FinalPiece)]
    P5,
    #[subenum(FinalPiece)]
    P6,
    #[subenum(FinalPiece)]
    P7,
    #[subenum(InterPiece)]
    P8,
    #[subenum(FinalPiece)]
    P9,
}

impl std::fmt::Display for FinalPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalPiece::P5 => write!(f, "P5"),
            FinalPiece::P6 => write!(f, "P6"),
            FinalPiece::P7 => write!(f, "P7"),
            FinalPiece::P9 => write!(f, "P9"),
        }
    }
}

impl TryFrom<&str> for FinalPiece {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "P5" => Ok(FinalPiece::P5),
            "P6" => Ok(FinalPiece::P6),
            "P7" => Ok(FinalPiece::P7),
            "P9" => Ok(FinalPiece::P9),
            _ => Err("Invalid work piece"),
        }
    }
}
