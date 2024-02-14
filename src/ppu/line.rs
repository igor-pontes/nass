use Line::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Line {
    PreRender,
    Render(usize),
    PostRender(usize)
}

impl Line {
    pub fn get(self) -> usize {
        match self {
            PreRender => 261,
            Render(line) => line,
            PostRender(line) => line,
        }
    }

    pub fn next(&mut self, dot: &mut usize) {
        *dot += 1;
        let inc = if *dot == 341 { *dot = 0; 1 } else { 0 };
        match self {
            PreRender => {
                if inc == 1 {  *self = Render(0); return; }
                *self = PreRender;
            },
            Render(line) => {
                let line = *line + inc;
                if line == 240 { *self = PostRender(line); return; }
                *self = Render(line);
            },
            PostRender(line) => {
                let line = *line + inc;
                if line == 261 { *self = PreRender; return; }
                *self = PostRender(line)
            },
        }
    }
}
