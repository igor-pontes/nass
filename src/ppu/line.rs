use Line::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Line {
    PreRender,
    Render(usize),
    PostRender(usize)
}

impl Line {
    pub fn get_line(self) -> usize {
        match self {
            PreRender => 261,
            Render(line) => line,
            PostRender(line) => line,
        }
    }

    pub fn next(self, rendering: bool, dot: &mut usize, even_frame: bool) -> Self {
        *dot += 1;
        let inc = if *dot == 341 { *dot = 0; 1 } else { 0 };
        match self {
            PreRender => {
                if inc == 1 { return Render(0); }
                if (*dot - 1) == 340 && rendering && even_frame { return Render(0); }
                PreRender
            },
            Render(line) => {
                let line = line + inc;
                if line == 240 { return PostRender(line); }
                Render(line)
            },
            PostRender(line) => {
                let line = line + inc;
                if line == 261 { return PreRender; }
                PostRender(line)
            },
        }
    }
}
