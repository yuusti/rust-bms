pub struct Bar {
    num: i32,
    pub ch: i32,
    pub notes: Vec<(f64, i32)>
}

impl Bar {
    fn pos(num: i32, obj: &str) -> Vec<(f64, i32)> {
        let num = num as f64;
        // TODO: implement
        return vec![(num, 1), (num + 0.25, 1), (num + 0.5, 1), (num + 0.75, 1)];
    }

    pub fn new(num: i32, ch: i32, obj: &str) -> Bar {
        Bar { num: num, ch: ch, notes: Bar::pos(num, &obj) }
    }
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

pub struct FixtureLoader {}

impl BmsLoader for FixtureLoader {
    fn load(&self) -> Chart {
        let mut v = vec![];

        let v2 = vec![11, 12, 13, 14, 15, 16, 17, 18];

        for i in 0..1000 {
            v.push(Bar::new(
                i as i32,
                v2[i % v2.len()],
                "01010101"
            ));
        }

        Chart {
            bpm: 130.0,
            bars: v
        }
    }
}

