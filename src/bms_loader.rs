pub struct Bar {
    pub num: i32,
    pub ch: i32,
    pub obj: String,
}

pub struct Chart {
    pub bpm: f64,
    pub bars: Vec<Bar>,
}

pub trait BmsLoader {
    fn load(&self) -> Chart;
}

pub struct BmsFileLoader {
    path: String
}

impl BmsLoader for BmsFileLoader {
    fn load(&self) -> Chart {
        unimplemented!()
    }
}

pub struct FixtureLoader {
}

impl BmsLoader for FixtureLoader {
    fn load(&self) -> Chart {
        Chart {
            bpm: 130.0,
            bars: vec![
                Bar {
                    num: 1,
                    ch: 11,
                    obj: "01".to_string(),
                },
                Bar {
                    num: 2,
                    ch: 11,
                    obj: "01".to_string(),
                },
                Bar {
                    num: 3,
                    ch: 11,
                    obj: "01".to_string(),
                }
            ]
        }
    }
}

