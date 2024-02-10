use Line::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Line {
    PreRender,
    Render(usize),
    PostRender(usize)
}

impl Line {
    pub fn next(self, rendering: bool, dot: &mut usize, even_frame: &mut bool, ) -> Self {
        *dot += 1;
        let old_dot = *dot;
        let inc = if *dot == 341 { *dot = 0; 1 } else { 0 };
        match self {
            PreRender => {
                if old_dot == 341 { return Render(0); }
                if old_dot == 340 && rendering && !(*even_frame) { return Render(0); }
                PreRender
            },
            Render(line) => {
                let line = line + inc;
                if line == 240 { return PostRender(line); }
                Render(line)
            },
            PostRender(line) => {
                let line = line + inc;
                if line == 261 { 
                    *even_frame = !(*even_frame);
                    return PreRender; 
                }
                PostRender(line)
            },
        }
    }
}
