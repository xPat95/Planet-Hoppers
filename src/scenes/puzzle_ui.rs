//! Illustrated, resolution-aware paper theatre. Hit regions and art share one layout.
use crate::minigames::southpark::{
    encounters::MEMBERS,
    puzzles::*,
    session::{SaveKennySession, SessionState},
    threats::ThreatKind,
};
use raylib::prelude::*;
use std::path::{Path, PathBuf};

const CREAM: Color = Color::new(245, 235, 212, 255);
const PAPER: Color = Color::new(255, 248, 232, 255);
const INK: Color = Color::new(73, 56, 45, 255);
const BROWN: Color = Color::new(167, 122, 81, 255);
const RED: Color = Color::new(201, 90, 78, 255);
const BLUE: Color = Color::new(114, 142, 159, 255);
const GREEN: Color = Color::new(122, 156, 120, 255);
const GOLD: Color = Color::new(233, 191, 98, 255);
const SHADOW: Color = Color::new(73, 56, 45, 32);

pub fn tr(es: bool, en: &str, spanish: &str) -> String {
    if es { spanish } else { en }.into()
}
#[derive(Clone, Copy)]
pub struct Layout {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub offset: Vector2,
    pub scene: Rectangle,
}
impl Layout {
    pub fn new(w: i32, h: i32) -> Self {
        let compact = w < 1000 || h < 650;
        let (width, height, side) = if compact {
            (960.0, 720.0, 258.0)
        } else {
            (1280.0, 800.0, 316.0)
        };
        let scale = (w as f32 / width).min(h as f32 / height);
        Self {
            width,
            height,
            scale,
            offset: Vector2::new(
                (w as f32 - width * scale) * 0.5,
                (h as f32 - height * scale) * 0.5,
            ),
            scene: Rectangle::new(side, 112.0, width - side - 28.0, height - 204.0),
        }
    }
    pub fn mouse(&self, p: Vector2) -> Vector2 {
        (p - self.offset) / self.scale
    }
    pub fn point(&self, x: f32, y: f32) -> Vector2 {
        Vector2::new(
            self.scene.x + x * self.scene.width,
            self.scene.y + y * self.scene.height,
        )
    }
    pub fn audio(&self) -> Rectangle {
        Rectangle::new(self.width - 296.0, 25.0, 166.0, 46.0)
    }
    pub fn pause(&self) -> Rectangle {
        Rectangle::new(self.width - 118.0, 25.0, 90.0, 46.0)
    }
    pub fn restart(&self) -> Rectangle {
        Rectangle::new(self.scene.x, self.height - 70.0, 154.0, 44.0)
    }
    pub fn back(&self) -> Rectangle {
        Rectangle::new(self.width - 222.0, self.height - 70.0, 194.0, 44.0)
    }
    pub fn primary(&self) -> Rectangle {
        Rectangle::new(
            self.scene.x + self.scene.width * 0.5 - 120.0,
            self.height - 70.0,
            240.0,
            44.0,
        )
    }
    pub fn cell(&self, i: usize) -> Rectangle {
        let size = self.scene.height * 0.139;
        let p = self.point(0.11, 0.25);
        Rectangle::new(
            p.x + (i % 4) as f32 * size,
            p.y + (i / 4) as f32 * size,
            size - 3.0,
            size - 3.0,
        )
    }
    pub fn light(&self, i: usize) -> Vector2 {
        self.point(
            0.27 + i as f32 * 0.23,
            0.35,
        )
    }
    pub fn lever(&self, i: usize) -> Rectangle {
        let p = self.point(0.13 + i as f32 * 0.205, 0.63);
        Rectangle::new(p.x, p.y, self.scene.width * 0.15, self.scene.height * 0.25)
    }
    pub fn key(&self, i: usize, errors: u8) -> Rectangle {
        let p = self.point(0.15 + i as f32 * 0.14, 0.30 + errors as f32 * 0.035);
        Rectangle::new(
            p.x,
            p.y,
            self.scene.width * 0.14 - 4.0,
            self.scene.height * 0.24,
        )
    }
    pub fn hole(&self, i: usize) -> Vector2 {
        let p = route_point(HOLES[i]);
        self.point(p.0, p.1)
    }
    pub fn case_grid(&self)->Rectangle {
        let side=(self.scene.height*0.72).min(self.scene.width*0.48);
        Rectangle::new(self.scene.x+self.scene.width*0.035,self.scene.y+self.scene.height*0.16,side,side)
    }
    pub fn case_cell(&self,cell:usize)->Rectangle {
        let g=self.case_grid();let size=g.width/6.;
        Rectangle::new(g.x+(cell%6) as f32*size,g.y+(cell/6) as f32*size,size,size)
    }
    pub fn case_panel(&self)->Rectangle {
        let g=self.case_grid();let x=g.x+g.width+20.;
        Rectangle::new(x,self.scene.y+self.scene.height*0.14,self.scene.x+self.scene.width-x-12.,self.scene.height*0.70)
    }
    pub fn case_card(&self,piece:usize)->Rectangle {
        let r=self.case_panel();let weights=[1.6,0.8,0.8,0.8,1.2,1.6];
        let unit=self.scene.height*0.66/6.8;
        Rectangle::new(r.x,r.y+weights[..piece].iter().sum::<f32>()*unit,r.width,weights[piece]*unit-1.)
    }
    pub fn case_accusation(&self,piece:usize)->Rectangle {
        let r=self.case_panel();let w=r.width/5.;
        Rectangle::new(r.x+(piece-1) as f32*w,self.scene.y+self.scene.height*0.86,w-3.,32.)
    }
    pub fn case_mode(&self)->Rectangle {let g=self.case_grid();Rectangle::new(g.x,self.scene.y+self.scene.height*0.895,g.width,24.)}
    pub fn bus_remove(&self)->Rectangle {let p=self.point(0.31,0.94);Rectangle::new(p.x,p.y,self.scene.width*0.23,30.)}
    pub fn bus_clear(&self)->Rectangle {let p=self.point(0.06,0.94);Rectangle::new(p.x,p.y,self.scene.width*0.21,30.)}
    pub fn bus_confirm(&self)->Rectangle {let p=self.point(0.69,0.94);Rectangle::new(p.x,p.y,self.scene.width*0.25,30.)}
    pub fn hit(&self, p: Vector2, run: &PuzzleRun) -> Option<usize> {
        match &run.board {
            Board::Heaters(_) => (0..16).find(|i| self.cell(*i).check_collision_point_rec(p)),
            Board::Ufo(_) => {
                (0..3).find(|i| (self.light(*i) - p).length() < self.scene.width * 0.047)
            }
            Board::Bus(_) => (0..36).find(|i|self.case_cell(*i).check_collision_point_rec(p)),
            Board::Piano(k) => (0..5).find(|i| self.key(*i, k.errors).check_collision_point_rec(p)),
            Board::Sewers(_) => (0..12).find(|i| (self.hole(*i) - p).length() < 24.0),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Start,
    Hit(usize),
    Restart,
    Pause,
    Back,
    Mute,
    SelectBus(usize),
    AnnotateBus(usize),
    ToggleBusMode,
    PlaceBus(usize),
    AccuseBus(usize),
    RemoveBus,
    ClearBus,
    ConfirmBus,
}
pub struct PuzzleUi {
    bus_press: Option<(usize,usize,std::time::Instant)>,
    bus_fixed: Option<(usize,std::time::Instant)>,
    title: Font,
    body: Font,
    kenny: Texture2D,
    pub muted: bool,
    pub(super) preference: PathBuf,
}
impl PuzzleUi {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut chars: String = (32..=255).filter_map(char::from_u32).collect();
        chars.push_str("•“”’—");
        let title = rl.load_font_ex(
            thread,
            &root
                .join("assets/fonts/GloriaHallelujah.ttf")
                .to_string_lossy(),
            48,
            Some(&chars),
        )?;
        let body = rl.load_font_ex(
            thread,
            &root
                .join("assets/fonts/TitilliumWeb-Bold.ttf")
                .to_string_lossy(),
            32,
            Some(&chars),
        )?;
        title
            .texture()
            .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        body.texture()
            .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let kenny = rl.load_texture(
            thread,
            &root
                .join("assets/textures/minigames/South Park/KennyMcCormick.png")
                .to_string_lossy(),
        )?;
        let preference = root.join("saves/save-kenny-music.txt");
        let muted = std::fs::read_to_string(&preference).is_ok_and(|s| s.trim() == "muted");
        Ok(Self {
            bus_press: None,
            bus_fixed: None,
            title,
            body,
            kenny,
            muted,
            preference,
        })
    }
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if let Some(parent) = self.preference.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&self.preference, if self.muted { "muted" } else { "on" }) {
            eprintln!("PUZZLE_AUDIO_PREFERENCE {e}");
        }
    }
    pub fn input(&mut self, rl: &RaylibHandle, s: &SaveKennySession) -> Action {
        if !s.puzzle.as_ref().is_some_and(|p|p.interactive()) {self.bus_press=None;self.bus_fixed=None;}
        let Some(run) = &s.puzzle else {
            return Action::None;
        };
        if !run.ready() {
            return Action::None;
        }
        let l = Layout::new(rl.get_screen_width(), rl.get_screen_height());
        let mouse = l.mouse(rl.get_mouse_position());
        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        if clicked && l.audio().check_collision_point_rec(mouse) {
            return Action::Mute;
        }
        if run.result.is_some() && run.result_time < 0.8 {
            return Action::None;
        }
        let reacting = matches!(
            s.state,
            SessionState::DeathAnimation | SessionState::DeathDialogue
        );
        if reacting {
            return Action::None;
        }
        if run.result.is_none()
            && (rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
                || (clicked && l.pause().check_collision_point_rec(mouse)))
        {
            return Action::Pause;
        }
        if clicked && l.back().check_collision_point_rec(mouse) {
            return if run.paused || run.result.is_some() {
                Action::Back
            } else {
                Action::Pause
            };
        }
        if run.paused {
            if clicked && l.primary().check_collision_point_rec(mouse) {
                return Action::Pause;
            }
            return Action::None;
        }
        if run.result.is_some() {
            if clicked && l.primary().check_collision_point_rec(mouse) && s.elapsed > 0.6 {
                return Action::Back;
            }
            if run.result == Some(false)
                && s.lives > 0
                && clicked
                && l.restart().check_collision_point_rec(mouse)
            {
                return Action::Restart;
            }
            return Action::None;
        }
        if clicked && l.restart().check_collision_point_rec(mouse) {
            return Action::Restart;
        }
        if !run.started {
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                || clicked && l.primary().check_collision_point_rec(mouse)
            {
                return Action::Start;
            }
            return Action::None;
        }
        if matches!(run.board,Board::Ufo(_)) {
            for (i,key) in [KeyboardKey::KEY_A,KeyboardKey::KEY_S,KeyboardKey::KEY_D].into_iter().enumerate() {
                if rl.is_key_pressed(key) {return Action::Hit(i);}
            }
            return Action::None;
        }
        if let Board::Bus(bus)=&run.board {
            use crate::minigames::southpark::murdoku as m;
            if clicked && l.case_mode().check_collision_point_rec(mouse) {self.bus_press=None;return Action::ToggleBusMode;}
            if clicked && l.bus_clear().check_collision_point_rec(mouse) {self.bus_press=None;return Action::ClearBus;}
            if rl.is_key_pressed(KeyboardKey::KEY_DELETE) || clicked && l.bus_remove().check_collision_point_rec(mouse) {self.bus_press=None;return Action::RemoveBus;}
            if bus.ready() && (rl.is_key_pressed(KeyboardKey::KEY_ENTER) || clicked && l.bus_confirm().check_collision_point_rec(mouse)) {self.bus_press=None;return Action::ConfirmBus;}
            let cell=(0..36).find(|i|l.case_cell(*i).check_collision_point_rec(mouse));
            if clicked {
                self.bus_press=None;
                if let Some(i)=(0..6).find(|i|l.case_card(*i).check_collision_point_rec(mouse)) {return Action::SelectBus(i);}
                if let Some(i)=(1..6).find(|i|l.case_accusation(*i).check_collision_point_rec(mouse)) {return Action::AccuseBus(i);}
                if let Some(c)=cell {
                    if bus.cross_mode {return Action::AnnotateBus(c);}
                    if let Some(piece)=bus.selected {if m::valid_cell(c) {self.bus_press=Some((c,piece,std::time::Instant::now()));}}
                    else if let Some(piece)=bus.positions.iter().position(|p|*p==Some(c)) {return Action::SelectBus(piece);}
                }
            }
            if let Some((c,piece,start))=self.bus_press {
                let released=rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);
                if cell!=Some(c) || bus.selected!=Some(piece) || (!released && !rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)) {self.bus_press=None;}
                else if m::is_long_press(start.elapsed().as_secs_f32()) {
                    self.bus_press=None;
                    if bus.can_place(piece,c) {
                        self.bus_fixed=Some((c,std::time::Instant::now()));return Action::PlaceBus(c);
                    }
                } else if released {self.bus_press=None;return Action::AnnotateBus(c);}
            }
            return Action::None;
        }
        if clicked {
            if let Some(i) = l.hit(mouse, run) {
                return Action::Hit(i);
            }
        }
        Action::None
    }
    fn text(&self, d: &mut impl RaylibDraw, text: &str, p: Vector2, size: f32, color: Color) {
        d.draw_text_ex(&self.body, text, p, size, 0.4, color);
    }
    fn heading(&self, d: &mut impl RaylibDraw, text: &str, p: Vector2, size: f32, color: Color) {
        d.draw_text_ex(&self.title, text, p, size * 1.40, 0.3, color);
    }
    fn wrap(
        &self,
        d: &mut impl RaylibDraw,
        text: &str,
        p: Vector2,
        width: f32,
        size: f32,
        color: Color,
    ) -> f32 {
        let mut y = p.y;
        for paragraph in text.split('\n') {
            let mut line = String::new();
            for word in paragraph.split_whitespace() {
                let candidate = if line.is_empty() {
                    word.into()
                } else {
                    format!("{line} {word}")
                };
                if self.body.measure_text(&candidate, size, 0.4).x > width && !line.is_empty() {
                    self.text(d, &line, Vector2::new(p.x, y), size, color);
                    y += size * 1.22;
                    line = word.into();
                } else {
                    line = candidate;
                }
            }
            self.text(d, &line, Vector2::new(p.x, y), size, color);
            y += size * 1.22;
        }
        y
    }
    fn button(
        &self,
        d: &mut impl RaylibDraw,
        r: Rectangle,
        label: &str,
        mouse: Vector2,
        down: bool,
        accent: Color,
        enabled: bool,
    ) {
        let hover = enabled && r.check_collision_point_rec(mouse);
        let mut face = r;
        if hover && down {
            face.y += 2.0;
        }
        paper(
            d,
            face,
            if enabled {
                if hover {
                    GOLD
                } else {
                    accent
                }
            } else {
                CREAM
            },
            INK,
        );
        let size = if label.chars().count() > 19 {
            24.0
        } else {
            28.0
        };
        let measure = self.body.measure_text(label, size, 0.4);
        self.text(
            d,
            label,
            Vector2::new(
                face.x + (face.width - measure.x) * 0.5,
                face.y + (face.height - size) * 0.5 - 2.0,
            ),
            size,
            if enabled { INK } else { BROWN },
        );
    }
    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        s: &SaveKennySession,
        portraits: &[Texture2D],
        death: Option<&Texture2D>,
        w: i32,
        h: i32,
        mouse: Vector2,
        down: bool,
    ) {
        let Some(run) = &s.puzzle else {
            return;
        };
        let es = s.adventure.as_ref().is_some_and(|a| a.spanish);
        let l = Layout::new(w, h);
        let mouse = l.mouse(mouse);
        let cam = Camera2D {
            offset: l.offset,
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: l.scale,
        };
        let mut d = d.begin_mode2D(cam);
        d.draw_rectangle(-4096, -4096, 8192, 8192, Color::new(43, 49, 47, 255));
        paper(
            &mut d,
            Rectangle::new(8.0, 8.0, l.width - 16.0, l.height - 16.0),
            CREAM,
            BROWN,
        );
        // Fine binding and quiet printed grain; never a heavy cardboard border.
        for y in (96..l.height as i32 - 30).step_by(18) {
            d.draw_line(20, y, 25, y, Color::new(167, 122, 81, 50));
        }
        self.heading(&mut d, "Save Kenny!", Vector2::new(30.0, 20.0), 34.0, INK);
        let title = title(run.kind, es);
        self.heading(
            &mut d,
            &title,
            Vector2::new(l.scene.x, 27.0),
            if l.width < 1100.0 { 26.0 } else { 32.0 },
            INK,
        );
        d.draw_line_ex(
            Vector2::new(30.0, 91.0),
            Vector2::new(l.width - 30.0, 91.0),
            1.5,
            BROWN,
        );
        self.button(
            &mut d,
            l.audio(),
            &tr(
                es,
                if self.muted {
                    "Music: OFF"
                } else {
                    "Music: ON"
                },
                if self.muted {
                    "Música: NO"
                } else {
                    "Música: SÍ"
                },
            ),
            mouse,
            down,
            PAPER,
            true,
        );
        self.button(
            &mut d,
            l.pause(),
            &tr(es, "Pause", "Pausa"),
            mouse,
            down,
            PAPER,
            run.result.is_none(),
        );
        let left = Rectangle::new(28.0, 112.0, l.scene.x - 48.0, l.scene.height);
        self.text(
            &mut d,
            &format!("{} 0{} / 05", tr(es, "CASE", "CASO"), run.group + 1),
            Vector2::new(left.x + 8.0, left.y + 5.0),
            17.0,
            BROWN,
        );
        self.heading(
            &mut d,
            &tr(es, "The rescue plan", "Plan de rescate"),
            Vector2::new(left.x + 8.0, left.y + 32.0),
            if l.width < 1100.0 { 23.0 } else { 26.0 },
            INK,
        );
        let instruction = if l.width < 1100.0 {
            short_instructions(run.kind, es)
        } else {
            instructions(run.kind, es)
        };
        self.wrap(
            &mut d,
            &instruction,
            Vector2::new(left.x + 8.0, left.y + 83.0),
            left.width - 14.0,
            if run.kind==ThreatKind::Bus {20.0} else if l.width < 1100.0 { 30.0 } else { 26.0 },
            INK,
        );
        let status_y = left.y + if l.width < 1100.0 { 257.0 } else { 266.0 };
        let multi_status=matches!(run.board,Board::Ufo(_)|Board::Bus(_)|Board::Piano(_));
        self.text(
            &mut d,
            &status(run, es),
            Vector2::new(left.x + 8.0, status_y),
            if multi_status {20.0} else if l.width < 1100.0 { 26.0 } else { 25.0 },
            if run.mistake && run.feedback > 0.0 {
                RED
            } else {
                INK
            },
        );
        if run.kind!=ThreatKind::Bus {meter(
            &mut d,
            Rectangle::new(left.x + 8.0, status_y + if multi_status {51.0}else{36.0}, left.width - 20.0, 10.0),
            progress(run),
            accent(run.kind),
        );}
        let portrait_y = status_y + 71.0;
        let portrait_h = (left.y + left.height - 66.0 - portrait_y).max(116.0);
        let members = MEMBERS[run.group];
        for (j, index) in members.iter().enumerate() {
            let width = left.width / members.len() as f32;
            let r = Rectangle::new(left.x + j as f32 * width, portrait_y, width, portrait_h);
            portrait(&mut d, &portraits[*index], r, 0.0, Color::WHITE);
        }
        self.wrap(
            &mut d,
            &aside(run.kind, es),
            Vector2::new(left.x + 5.0, left.y + left.height - 53.0),
            left.width - 10.0,
            19.0,
            BROWN,
        );
        paper(&mut d, l.scene, PAPER, BROWN);
        // Clip art to its own illustrated window, using the same transform as hit tests.
        unsafe {
            raylib::ffi::BeginScissorMode(
                (l.offset.x + l.scene.x * l.scale) as i32,
                (l.offset.y + l.scene.y * l.scale) as i32,
                (l.scene.width * l.scale) as i32,
                (l.scene.height * l.scale) as i32,
            );
        }
        match &run.board {
            Board::Heaters(board) => self.heaters(&mut d, &l, run, board, mouse, es),
            Board::Ufo(board) => self.ufo(&mut d, &l, run, board, mouse, es),
            Board::Bus(board) => self.bus(&mut d, &l, run, board, mouse, es),
            Board::Piano(board) => self.piano(&mut d, &l, run, board, mouse, es),
            Board::Sewers(board) => self.sewers(&mut d, &l, run, board, mouse, es),
        }
        if run.result == Some(true) && run.result_time < 1.0 {
            let (x, y) = match run.kind {
                ThreatKind::Snowman => (0.86, 0.65),
                ThreatKind::Ufo => (0.5, 0.67),
                ThreatKind::Bus => (0.8, 0.17),
                ThreatKind::Piano => (0.5, 0.72),
                ThreatKind::Sewer => (0.93, 0.68),
            };
            let centre = l.point(x, y);
            for i in 0..5 {
                let angle = i as f32 * std::f32::consts::TAU / 5.0;
                let p = centre
                    + Vector2::new(angle.cos(), angle.sin()) * (35.0 + run.result_time * 28.0);
                let color = Color::new(233, 191, 98, ((1.0 - run.result_time) * 255.0) as u8);
                d.draw_line_ex(
                    p - Vector2::new(5.0, 0.0),
                    p + Vector2::new(5.0, 0.0),
                    3.0,
                    color,
                );
                d.draw_line_ex(
                    p - Vector2::new(0.0, 5.0),
                    p + Vector2::new(0.0, 5.0),
                    3.0,
                    color,
                );
            }
        }
        unsafe {
            raylib::ffi::EndScissorMode();
        }
        d.draw_rectangle_rounded_lines_ex(l.scene, 0.025, 8, 1.5, BROWN);
        self.text(
            &mut d,
            &format!(
                "{}: {}    {}: {}/5",
                tr(es, "Lives", "Vidas"),
                s.lives,
                tr(es, "Rescues", "Rescates"),
                s.saves
            ),
            Vector2::new(36.0, l.height - 61.0),
            20.0,
            INK,
        );
        let reacting = matches!(
            s.state,
            SessionState::DeathAnimation | SessionState::DeathDialogue
        );
        self.button(
            &mut d,
            l.restart(),
            &tr(es, "Reset", "Reiniciar"),
            mouse,
            down,
            PAPER,
            !reacting && run.result != Some(true) && s.lives > 0 && !run.paused,
        );
        self.button(
            &mut d,
            l.back(),
            &tr(es, "Back to town", "Volver al pueblo"),
            mouse,
            down,
            PAPER,
            !reacting,
        );
        if !run.started || run.paused || run.result.is_some() {
            let label = if run.paused {
                tr(es, "Resume", "Continuar")
            } else if run.result.is_some() {
                tr(es, "Continue", "Continuar")
            } else {
                tr(es, "Let's save Kenny!", "¡A salvar a Kenny!")
            };
            self.button(
                &mut d,
                l.primary(),
                &label,
                mouse,
                down,
                GOLD,
                !reacting && run.ready(),
            );
        }
        if run.paused {
            self.result_banner(
                &mut d,
                &l,
                &tr(es, "A little intermission", "Un pequeño descanso"),
                &tr(
                    es,
                    "The puzzle is paused. Return to town to leave this mission unfinished.",
                    "El puzzle está en pausa. Volver al pueblo deja esta misión pendiente.",
                ),
                None,
            );
        } else if run.result.is_some()
            && run.result_time > 0.8
            && !matches!(s.state, SessionState::DeathAnimation)
        {
            if s.state == SessionState::DeathDialogue {
                let (name, text, count) = s.dialogue();
                let text = if es {
                    if name == "Stan" {
                        "¡Dios mío! ¡Mataron a Kenny!"
                    } else {
                        "¡Malditos bastardos!"
                    }
                } else {
                    text
                };
                self.result_banner(
                    &mut d,
                    &l,
                    name,
                    &text.chars().take(count).collect::<String>(),
                    death,
                );
            } else {
                self.result_banner(
                    &mut d,
                    &l,
                    &if run.result == Some(true) {
                        tr(es, "Kenny lives another day!", "¡Kenny vive un día más!")
                    } else {
                        tr(es, "Oh no, Kenny...", "Ay, Kenny...")
                    },
                    &if run.result == Some(true) {
                        tr(
                            es,
                            "Rescue complete. Your friends will remember this.",
                            "Rescate completado. Tus amigos no lo olvidarán.",
                        )
                    } else if s.lives > 0 {
                        tr(
                            es,
                            "One life lost. Reset for another try, or return to town.",
                            "Perdiste una vida. Reinicia para reintentar o vuelve al pueblo.",
                        )
                    } else {
                        tr(
                            es,
                            "No lives left. Continue to the results.",
                            "No quedan vidas. Continúa para ver el resultado.",
                        )
                    },
                    None,
                );
            }
        }
        let opacity = 1.0 - run.transition / FADE_SECONDS;
        if opacity > 0.0 {
            d.draw_rectangle(
                0,
                0,
                l.width as i32,
                l.height as i32,
                Color::new(245, 235, 212, (opacity * 255.0) as u8),
            );
        }
    }
    fn result_banner(
        &self,
        d: &mut impl RaylibDraw,
        l: &Layout,
        title: &str,
        body: &str,
        portrait_image: Option<&Texture2D>,
    ) {
        let r = Rectangle::new(
            l.scene.x + 24.0,
            l.scene.y + l.scene.height * 0.32,
            l.scene.width - 48.0,
            180.0,
        );
        paper(d, r, PAPER, BROWN);
        let x = if let Some(t) = portrait_image {
            portrait(
                d,
                t,
                Rectangle::new(r.x + 8.0, r.y + 10.0, 110.0, 155.0),
                0.0,
                Color::WHITE,
            );
            r.x + 128.0
        } else {
            r.x + 24.0
        };
        self.heading(
            d,
            title,
            Vector2::new(x, r.y + 16.0),
            if l.width < 1100.0 { 25.0 } else { 30.0 },
            INK,
        );
        self.wrap(
            d,
            body,
            Vector2::new(x, r.y + 76.0),
            r.x + r.width - x - 20.0,
            23.0,
            INK,
        );
    }
    fn kenny(&self, d: &mut impl RaylibDraw, feet: Vector2, height: f32, roll: f32, tint: Color) {
        let width = height * self.kenny.width as f32 / self.kenny.height as f32;
        d.draw_ellipse(
            feet.x as i32,
            (feet.y + 2.0) as i32,
            width * 0.43,
            5.0,
            SHADOW,
        );
        d.draw_texture_pro(
            &self.kenny,
            Rectangle::new(0.0, 0.0, self.kenny.width as f32, self.kenny.height as f32),
            Rectangle::new(feet.x, feet.y, width, height),
            Vector2::new(width * 0.5, height),
            roll,
            tint,
        );
    }
    fn ufo(
        &self,
        d: &mut impl RaylibDraw,
        l: &Layout,
        run: &PuzzleRun,
        u: &Ufo,
        mouse: Vector2,
        es: bool,
    ) {
        let b = l.scene;
        d.draw_rectangle_rec(b, Color::new(43, 44, 72, 255));
        winter(d, l, true);
        for i in 0..35 {
            let p = l.point(
                ((i * 37 + 11) % 97) as f32 / 100.0,
                ((i * 17 + 3) % 47) as f32 / 100.0,
            );
            d.draw_circle_v(
                p,
                if i % 4 == 0 { 2.0 } else { 1.0 },
                Color::new(245, 235, 212, 160),
            );
        }
        d.draw_circle_v(l.point(0.88, 0.13), 28.0, PAPER);
        d.draw_circle_v(l.point(0.90, 0.11), 26.0, Color::new(43, 44, 72, 255));
        let cx = l.point(0.5, 0.29);
        let disc_w = b.width * 0.43;
        let beam = if run.result == Some(true) {
            0
        } else {
            (50.0 + u.charge * 100.0) as u8
        };
        d.draw_triangle(
            l.point(0.5, 0.35),
            l.point(0.28, 0.93),
            l.point(0.72, 0.93),
            Color::new(165, 227, 182, beam),
        );
        d.draw_ellipse(
            cx.x as i32,
            (cx.y - b.height * 0.055) as i32,
            b.width * 0.23,
            b.height * 0.16,
            Color::new(137, 168, 172, 255),
        );
        d.draw_ellipse(
            (cx.x - b.width * 0.02) as i32,
            (cx.y - b.height * 0.07) as i32,
            b.width * 0.19,
            b.height * 0.12,
            Color::new(190, 219, 210, 255),
        );
        for i in 0..3 {
            let p = l.point(0.39 + i as f32 * 0.11, 0.14);
            d.draw_ellipse(
                p.x as i32,
                p.y as i32,
                12.0,
                17.0,
                Color::new(115, 157, 132, 255),
            );
            d.draw_ellipse((p.x - 4.0) as i32, (p.y - 1.0) as i32, 3.0, 6.0, INK);
            d.draw_ellipse((p.x + 4.0) as i32, (p.y - 1.0) as i32, 3.0, 6.0, INK);
        }
        d.draw_ellipse(
            cx.x as i32,
            (cx.y + 10.0) as i32,
            disc_w,
            b.height * 0.115,
            Color::new(38, 34, 55, 255),
        );
        d.draw_ellipse(
            cx.x as i32,
            cx.y as i32,
            disc_w,
            b.height * 0.105,
            Color::new(133, 119, 158, 255),
        );
        d.draw_ellipse(
            cx.x as i32,
            (cx.y - 9.0) as i32,
            disc_w * 0.94,
            b.height * 0.055,
            Color::new(176, 167, 192, 255),
        );
        for i in 0..3 {
            let p = l.light(i);
            let r = b.width * 0.041;
            let lit = run.started && run.result.is_none() && u.target == i && u.reaction>=0.;
            if lit {
                d.draw_circle_v(
                    p,
                    r + 8.0 + (run.animation * 6.0).sin() * 2.0,
                    Color::new(233, 239, 152, 65),
                );
            }
            d.draw_circle_v(p + Vector2::new(0.0, 3.0), r + 3.0, INK);
            d.draw_circle_v(
                p,
                r,
                if lit {
                    Color::new(229, 244, 140, 255)
                } else {
                    Color::new(79, 77, 101, 255)
                },
            );
            if lit {
                d.draw_circle_v(p - Vector2::new(r * 0.2, r * 0.25), r * 0.42, PAPER);
                let radius=r*(1.+(1.-u.reaction/u.window(run.elapsed))*1.3).max(0.7);
                d.draw_ring(p,radius,radius+3.,0.,360.,64,GOLD);
                d.draw_ring(p,r-2.,r,0.,360.,64,PAPER);
            }
            if run.interactive() && (p - mouse).length() < r + 5.0 {
                d.draw_ring(p, r + 2.0, r + 4.0, 0.0, 360.0, 32, PAPER);
            }
            self.text(
                d,
                ["A","S","D"][i],
                p - Vector2::new(6.0, 12.0),
                20.0,
                if lit { INK } else { PAPER },
            );
        }
        let lift = if run.result == Some(true) {
            0.0
        } else if run.result == Some(false) {
            (u.charge + run.result_time * 0.3).min(1.3)
        } else {
            u.charge
        };
        self.kenny(
            d,
            l.point(0.5, 0.88 - lift * 0.31),
            b.height * 0.24,
            (run.animation * 3.0).sin() * 3.0,
            Color::WHITE,
        );
        let plate = Rectangle::new(
            b.x + b.width * 0.18,
            b.y + b.height * 0.92,
            b.width * 0.64,
            28.0,
        );
        paper(
            d,
            plate,
            Color::new(60, 59, 82, 255),
            Color::new(170, 165, 188, 255),
        );
        meter(
            d,
            Rectangle::new(plate.x + 7.0, plate.y + 8.0, plate.width - 14.0, 12.0),
            u.charge,
            if u.charge > 0.65 { RED } else { GREEN },
        );
        self.text(
            d,
            &tr(es, "TRACTOR BEAM", "RAYO DE ABDUCCIÓN"),
            l.point(0.03, 0.04),
            18.0,
            PAPER,
        );
        if run.feedback > 0.0 {
            self.text(
                d,
                run.rating.unwrap_or(Rating::Miss).label(es),
                l.point(0.70, 0.64-run.feedback*0.025),
                24.0,
                GOLD,
            );
        }
    }
    fn case_token(&self,d:&mut impl RaylibDraw,piece:usize,r:Rectangle,selected:bool) {
        let colors=[GOLD,BLUE,RED,GREEN,Color::new(40,43,48,255),PAPER];
        if selected {d.draw_rectangle_lines_ex(r,3.,GOLD);}
        if piece==0 {self.kenny(d,Vector2::new(r.x+r.width*0.5,r.y+r.height*0.92),r.height*0.85,0.,Color::WHITE);return;}
        let c=Vector2::new(r.x+r.width*0.5,r.y+r.height*0.42);let size=r.width.min(r.height);
        d.draw_circle_v(c+Vector2::new(2.,3.),size*0.25,SHADOW);
        d.draw_rectangle_rounded(Rectangle::new(c.x-size*0.28,c.y+size*0.10,size*0.56,size*0.37),0.3,8,INK);
        d.draw_rectangle_rounded(Rectangle::new(c.x-size*0.25,c.y+size*0.13,size*0.50,size*0.30),0.3,8,colors[piece]);
        d.draw_circle_v(c,size*0.21,Color::new(233,187,143,255));
        d.draw_rectangle_rounded(Rectangle::new(c.x-size*0.25,c.y-size*0.26,size*0.5,size*0.19),0.3,6,colors[piece]);
        d.draw_line_ex(c+Vector2::new(-size*0.26,-size*0.08),c+Vector2::new(size*0.28,-size*0.08),2.,INK);
        self.text(d,["K","A","B","C","D","E"][piece],Vector2::new(c.x-5.,c.y+size*0.14),size*0.24,if piece==4 {PAPER}else{INK});
    }
    fn bus(&self,d:&mut impl RaylibDraw,l:&Layout,run:&PuzzleRun,bus:&Bus,mouse:Vector2,es:bool) {
        use crate::minigames::southpark::murdoku as m;
        let b=l.scene;let grid=l.case_grid();let size=grid.width/6.;
        d.draw_rectangle_rec(b,Color::new(225,226,202,255));
        self.heading(d,&tr(es,"Who Will Hit Kenny?","¿Quién atropellará a Kenny?"),l.point(0.025,0.015),if l.width<1100. {23.}else{29.},INK);
        let zones=[Color::new(176,198,209,255),Color::new(189,181,204,255),Color::new(163,174,154,255),Color::new(190,169,174,255)];
        for z in 0..4 {let p=l.point(0.025+z as f32*0.24,0.105);d.draw_circle_v(p+Vector2::new(5.,7.),5.,zones[z]);self.text(d,m::zone_name(z as u8,es),p+Vector2::new(14.,-2.),16.,INK);}
        paper(d,Rectangle::new(grid.x-5.,grid.y-5.,grid.width+10.,grid.height+10.),PAPER,INK);
        for cell in 0..36 {
            let r=l.case_cell(cell);d.draw_rectangle_rec(r,zones[m::ZONES[cell] as usize]);
            d.draw_rectangle_lines_ex(r,1.,Color::new(222,220,204,255));
            if m::BUILDINGS.contains(&cell) || cell==m::RADIO {
                let block=Rectangle::new(r.x+2.,r.y+2.,size-4.,size-4.);
                d.draw_rectangle_rec(block,Color::new(126,99,111,255));
                for dx in [0.25,0.63] {d.draw_rectangle_rec(Rectangle::new(r.x+size*dx,r.y+size*0.35,size*0.15,size*0.22),PAPER);}
                if cell==m::RADIO {d.draw_line_ex(Vector2::new(r.x+size*0.5,r.y+5.),Vector2::new(r.x+size*0.5,r.y+size*0.25),3.,INK);}
            }
            if m::KIOSKS.contains(&cell) {
                paper(d,Rectangle::new(r.x+7.,r.y+7.,size-14.,size-14.),BROWN,INK);
                for row in 1..4 {let y=r.y+size*row as f32/4.;d.draw_line_ex(Vector2::new(r.x+11.,y),Vector2::new(r.x+size-11.,y),2.,PAPER);}
            }
            if cell==m::PLANT {d.draw_circle_v(Vector2::new(r.x+size*0.5,r.y+size*0.5),size*0.29,GREEN);d.draw_rectangle_rec(Rectangle::new(r.x+size*0.3,r.y+size*0.6,size*0.4,size*0.2),BROWN);}
            if m::BENCHES.contains(&cell) {
                for row in 0..3 {d.draw_rectangle_rec(Rectangle::new(r.x+size*0.17,r.y+size*(0.25+row as f32*0.14),size*0.66,size*0.10),BROWN);}
                for x in [0.23,0.7] {d.draw_rectangle_rec(Rectangle::new(r.x+size*x,r.y+size*0.60,3.,size*0.18),INK);}
            }
            if m::MANHOLES.contains(&cell) {let c=Vector2::new(r.x+size*0.5,r.y+size*0.5);d.draw_ellipse(c.x as i32,c.y as i32,size*0.34,size*0.23,INK);d.draw_ellipse(c.x as i32,c.y as i32,size*0.24,size*0.14,Color::new(75,94,85,255));}
        }
        for pair in m::BUSES {let r=l.case_cell(pair[0]);school_bus(d,Vector2::new(r.x+5.,r.y+size*0.76),size*2.-10.);}
        // Join the three original desk cells into the same L-shaped building.
        for cell in m::BUILDINGS {let r=l.case_cell(cell);
            if cell%6<5 && m::BUILDINGS.contains(&(cell+1)) {d.draw_rectangle_rec(Rectangle::new(r.x+size-2.,r.y+2.,4.,size-4.),Color::new(126,99,111,255));}
            if cell+6<36 && m::BUILDINGS.contains(&(cell+6)) {d.draw_rectangle_rec(Rectangle::new(r.x+2.,r.y+size-2.,size-4.,4.),Color::new(126,99,111,255));}
        }
        // Exact region borders, including the L-shaped depot and narrow centre leg.
        for cell in 0..36 {let r=l.case_cell(cell);let z=m::ZONES[cell];
            if cell%6==0 || m::ZONES[cell-1]!=z {d.draw_line_ex(Vector2::new(r.x,r.y),Vector2::new(r.x,r.y+size),4.,INK);}
            if cell<6 || m::ZONES[cell-6]!=z {d.draw_line_ex(Vector2::new(r.x,r.y),Vector2::new(r.x+size,r.y),4.,INK);}
        }
        d.draw_rectangle_lines_ex(grid,4.,INK);
        for (piece,cell) in bus.positions.iter().enumerate() {if let Some(cell)=cell {let r=l.case_cell(*cell);self.case_token(d,piece,Rectangle::new(r.x+4.,r.y+4.,size-8.,size-8.),bus.selected==Some(piece));}}
        // Pencil marks are smaller outlined letters, never full-size final tokens.
        for cell in 0..36 {
            let r=l.case_cell(cell);
            for piece in 0..6 {if bus.candidates[cell]&(1<<piece)!=0 {
                let c=Vector2::new(r.x+size*(0.18+(piece%3) as f32*0.30),r.y+size*(0.58+(piece/3) as f32*0.28));
                let radius=(size*0.12).max(7.);d.draw_circle_v(c,radius,PAPER.alpha(0.9));
                d.draw_circle_lines(c.x as i32,c.y as i32,radius,INK);
                self.text(d,["K","A","B","C","D","E"][piece],c-Vector2::new(radius*0.55,radius*0.9),(radius*1.65).max(12.),INK);
            }}
            if bus.is_excluded(cell) {
                let inset=size*0.17;
                d.draw_line_ex(Vector2::new(r.x+inset,r.y+inset),Vector2::new(r.x+size-inset,r.y+size-inset),3.,INK.alpha(0.8));
                d.draw_line_ex(Vector2::new(r.x+size-inset,r.y+inset),Vector2::new(r.x+inset,r.y+size-inset),3.,INK.alpha(0.8));
            }
        }
        if run.interactive() {
            if let Some((cell,_,start))=self.bus_press {
                let r=l.case_cell(cell);let t=(start.elapsed().as_secs_f32()/m::HOLD_SECONDS).min(1.);
                d.draw_ring(Vector2::new(r.x+size*0.5,r.y+size*0.5),size*0.39,size*0.44,-90.,-90.+360.*t,40,GOLD);
            }
            if let Some((cell,at))=self.bus_fixed {if at.elapsed().as_secs_f32()<0.35 {d.draw_rectangle_lines_ex(l.case_cell(cell),4.,GOLD);}}
        }
        for piece in 0..6 {
            let r=l.case_card(piece);paper(d,r,if bus.selected==Some(piece){Color::new(250,236,193,255)}else{PAPER},if bus.selected==Some(piece){GOLD}else{BROWN});
            self.case_token(d,piece,Rectangle::new(r.x+3.,r.y+1.,18.,18.),false);
            self.text(d,m::name(piece,es),Vector2::new(r.x+25.,r.y+1.),16.,INK);
            let mut font=if l.width<1100. {18.0}else{20.0};
            loop {
                let mut lines=1;let mut row=String::new();
                for word in m::clue(piece,es).split_whitespace() {
                    let next=if row.is_empty(){word.into()}else{format!("{row} {word}")};
                    if self.body.measure_text(&next,font,0.4).x>r.width-12. && !row.is_empty() {lines+=1;row=word.into();}else{row=next;}
                }
                if lines as f32*font*1.22<=r.height-21. || font<=16. {break;}font-=0.5;
            }
            self.wrap(d,m::clue(piece,es),Vector2::new(r.x+6.,r.y+19.),r.width-12.,font,INK);
        }
        let panel=l.case_panel();
        self.button(d,l.case_mode(),&tr(es,if bus.cross_mode {"Mode: X (click to mark)"}else{"Mode: pencil / hold to fix"},if bus.cross_mode {"Modo: X (clic para marcar)"}else{"Modo: lápiz / mantén para fijar"}),mouse,false,if bus.cross_mode {GOLD}else{PAPER},run.interactive());
        self.text(d,&tr(es,"Dangerous driver?","¿Chofer peligroso?"),Vector2::new(panel.x,b.y+b.height*0.815),19.,INK);
        for piece in 1..6 {let r=l.case_accusation(piece);paper(d,r,if bus.accused==Some(piece){GOLD}else{PAPER},INK);self.case_token(d,piece,Rectangle::new(r.x+(r.width-r.height)*0.5,r.y,r.height,r.height),false);}
        for (r,label,enabled) in [(l.bus_clear(),tr(es,"Clear board","Vaciar tablero"),true),(l.bus_remove(),tr(es,"Remove token","Retirar ficha"),bus.selected.is_some()),(l.bus_confirm(),tr(es,"Confirm","Confirmar"),bus.ready())] {
            self.button(d,r,&label,mouse,false,if enabled{PAPER}else{Color::new(205,202,191,255)},run.interactive()&&enabled);
        }
        if run.result.is_some() {
            // Consequence only after the single confirmation; never disclose a
            // location or driver before the player commits their whole answer.
            let stage=Rectangle::new(panel.x,b.y+b.height*0.48,panel.width,b.height*0.25);
            paper(d,stage,PAPER,BROWN);let t=run.result_time.min(1.);
            let stop=if run.result==Some(true){0.25}else{0.62};
            self.kenny(d,Vector2::new(stage.x+stage.width*0.80,stage.y+stage.height*0.86),stage.height*0.67,if run.result==Some(false){t*85.}else{0.},Color::WHITE);
            school_bus(d,Vector2::new(stage.x+stage.width*(0.03+t*stop),stage.y+stage.height*0.80),stage.width*0.42);
        }
    }
    fn piano(
        &self,
        d: &mut impl RaylibDraw,
        l: &Layout,
        run: &PuzzleRun,
        p: &Piano,
        mouse: Vector2,
        es: bool,
    ) {
        let b = l.scene;
        d.draw_rectangle_rec(b, Color::new(235, 218, 187, 255));
        // A little outdoor theatre: scalloped curtain, rigging, floorboards.
        for i in 0..12 {
            let x = b.x + i as f32 * b.width / 11.0;
            d.draw_rectangle_rec(Rectangle::new(x, b.y, b.width / 11.0, b.height * 0.11), RED);
            d.draw_circle(
                (x + b.width / 22.0) as i32,
                (b.y + b.height * 0.10) as i32,
                b.width / 24.0,
                RED,
            );
        }
        d.draw_rectangle_rec(
            Rectangle::new(b.x, b.y, b.width * 0.07, b.height * 0.84),
            RED,
        );
        d.draw_rectangle_rec(
            Rectangle::new(b.x + b.width * 0.93, b.y, b.width * 0.07, b.height * 0.84),
            RED,
        );
        d.draw_rectangle_rec(
            Rectangle::new(b.x, b.y + b.height * 0.88, b.width, b.height * 0.12),
            BROWN,
        );
        for i in 0..12 {
            let x = b.x + i as f32 * b.width / 11.0;
            d.draw_line_ex(
                Vector2::new(x, b.y + b.height * 0.88),
                Vector2::new(x, b.y + b.height),
                1.0,
                INK,
            );
        }
        if run.result == Some(false) {
            self.kenny(d, l.point(0.50, 0.91), b.height * 0.25, 0.0, Color::WHITE);
        }
        let drop = if run.result == Some(false) {
            (run.result_time * 2.0).min(1.0) * b.height * 0.25
        } else {
            0.0
        };
        let top = l.point(0.12, 0.20 + p.errors as f32 * 0.035) + Vector2::new(0.0, drop);
        for x in [0.25, 0.75] {
            let a = l.point(x, 0.0);
            d.draw_line_ex(a, Vector2::new(a.x, top.y + 10.0), 6.0, INK);
            d.draw_line_ex(
                a + Vector2::new(1.0, 0.0),
                Vector2::new(a.x + 1.0, top.y + 10.0),
                2.0,
                GOLD,
            );
        }
        paper(
            d,
            Rectangle::new(top.x, top.y, b.width * 0.76, b.height * 0.37),
            Color::new(95, 64, 49, 255),
            INK,
        );
        self.heading(
            d,
            "South Park & Sons",
            Vector2::new(top.x + b.width * 0.12, top.y + 5.0),
            22.0,
            GOLD,
        );
        let colors = [BLUE, GREEN, GOLD, RED, Color::new(153, 130, 171, 255)];
        let lit = if p.demonstrating && run.started {
            p.lit()
        } else if run.feedback > 0.0 && !run.mistake {
            run.last_hit
        } else {
            None
        };
        for i in 0..5 {
            let mut key = l.key(i, p.errors);
            key.y += drop;
            let is_lit = lit == Some(i);
            let hover =
                run.interactive() && !p.demonstrating && key.check_collision_point_rec(mouse);
            let pressed = run.feedback > 0.0 && run.last_hit == Some(i) && !p.demonstrating;
            if pressed {
                key.y += 3.0;
                key.height -= 3.0;
            }
            paper(
                d,
                key,
                if is_lit {
                    colors[i]
                } else if hover {
                    CREAM
                } else {
                    PAPER
                },
                INK,
            );
            d.draw_rectangle_rec(
                Rectangle::new(
                    key.x + key.width * 0.26,
                    key.y + key.height * 0.74,
                    key.width * 0.48,
                    6.0,
                ),
                colors[i],
            );
            self.text(
                d,
                &(i + 1).to_string(),
                Vector2::new(key.x + key.width * 0.46, key.y + key.height * 0.81),
                20.0,
                INK,
            );
            if i < 4 {
                d.draw_rectangle_rounded(
                    Rectangle::new(key.x + key.width - 9.0, key.y, 22.0, key.height * 0.51),
                    0.1,
                    6,
                    INK,
                );
            }
        }
        if run.result != Some(false) {
            self.kenny(d, l.point(0.50, 0.91), b.height * 0.25, 0.0, Color::WHITE);
        }
        if run.result.is_some() {
            return;
        }
        let cue = if !run.started {
            tr(es, "The stage is yours.", "El escenario es tuyo.")
        } else if p.demonstrating {
            tr(es, "WATCH THE KEYS", "OBSERVA LAS TECLAS")
        } else {
            tr(es, "YOUR TURN", "TU TURNO")
        };
        self.text(
            d,
            &cue,
            l.point(0.14, 0.64),
            24.0,
            if p.demonstrating { BROWN } else { RED },
        );
        for i in 0..p.length() {
            let q = l.point(0.17 + i as f32 * 0.052, 0.73);
            d.draw_circle_v(
                q,
                7.0,
                if !p.demonstrating && i < p.input {
                    GREEN
                } else {
                    BROWN
                },
            );
        }
        if run.mistake && run.feedback > 0.0 {
            self.text(
                d,
                &tr(es, "Again, from the top!", "¡Otra vez, desde el principio!"),
                l.point(0.14, 0.81),
                20.0,
                RED,
            );
        }
    }
    fn sewers(
        &self,
        d: &mut impl RaylibDraw,
        l: &Layout,
        run: &PuzzleRun,
        s: &Sewers,
        mouse: Vector2,
        es: bool,
    ) {
        let b = l.scene;
        d.draw_rectangle_rec(b, Color::new(213, 225, 194, 255));
        // Street rows and shopfronts echo the town's grid; this is an illustrated
        // board, deliberately separate from the unchanged world minimap projection.
        for (i, (x, y)) in [
            (0.06, 0.07),
            (0.21, 0.07),
            (0.47, 0.07),
            (0.66, 0.07),
            (0.83, 0.07),
            (0.08, 0.34),
            (0.20, 0.61),
            (0.48, 0.65),
            (0.88, 0.57),
            (0.48, 0.86),
        ]
        .into_iter()
        .enumerate()
        {
            house(
                d,
                l.point(x, y),
                b.width * 0.072,
                [RED, BLUE, GREEN, BROWN][i % 4],
            );
        }
        let school = l.point(0.53, 0.26);
        let school_w = b.width * 0.19;
        d.draw_rectangle_rec(
            Rectangle::new(school.x + 3.0, school.y + 4.0, school_w, 56.0),
            SHADOW,
        );
        d.draw_rectangle_rec(
            Rectangle::new(school.x, school.y, school_w, 56.0),
            Color::new(219, 186, 130, 255),
        );
        d.draw_triangle(
            Vector2::new(school.x + school_w * 0.5, school.y - 31.0),
            Vector2::new(school.x - 8.0, school.y + 2.0),
            Vector2::new(school.x + school_w + 8.0, school.y + 2.0),
            RED,
        );
        for i in 0..5 {
            d.draw_rectangle_rec(
                Rectangle::new(
                    school.x + 12.0 + i as f32 * (school_w - 25.0) / 5.0,
                    school.y + 9.0,
                    14.0,
                    20.0,
                ),
                PAPER,
            );
            d.draw_rectangle_rec(
                Rectangle::new(
                    school.x + 15.0 + i as f32 * (school_w - 25.0) / 5.0,
                    school.y + 12.0,
                    8.0,
                    14.0,
                ),
                BLUE,
            );
        }
        d.draw_rectangle_rec(
            Rectangle::new(
                school.x + school_w * 0.43,
                school.y + 32.0,
                school_w * 0.14,
                24.0,
            ),
            INK,
        );
        self.text(
            d,
            &tr(es, "SCHOOL", "ESCUELA"),
            Vector2::new(school.x + school_w * 0.23, school.y - 17.0),
            17.0,
            PAPER,
        );
        let pond = l.point(0.87, 0.30);
        d.draw_ellipse(pond.x as i32, pond.y as i32, 42.0, 23.0, PAPER);
        d.draw_ellipse(pond.x as i32, pond.y as i32, 36.0, 18.0, BLUE);
        d.draw_line_ex(
            pond + Vector2::new(-19.0, -5.0),
            pond + Vector2::new(7.0, -5.0),
            2.0,
            PAPER,
        );
        d.draw_line_ex(
            pond + Vector2::new(-2.0, 5.0),
            pond + Vector2::new(21.0, 5.0),
            2.0,
            PAPER,
        );
        for (x, y) in [(0.17, 0.42), (0.48, 0.77), (0.86, 0.39)] {
            let p = l.point(x, y);
            for i in 0..6 {
                d.draw_rectangle_rec(Rectangle::new(p.x + i as f32 * 7.0, p.y, 3.0, 15.0), PAPER);
            }
            d.draw_line_ex(
                p + Vector2::new(0.0, 5.0),
                p + Vector2::new(38.0, 5.0),
                2.0,
                BROWN,
            );
        }
        for (x, y) in [
            (0.05, 0.59),
            (0.17, 0.89),
            (0.28, 0.91),
            (0.56, 0.35),
            (0.91, 0.37),
            (0.86, 0.92),
            (0.67, 0.97),
            (0.05, 0.95),
        ] {
            tree(d, l.point(x, y), b.height * 0.11);
        }
        let path: Vec<_> = SEWER_ROUTE.iter().map(|p| l.point(p.0, p.1)).collect();
        for pair in path.windows(2) {
            d.draw_line_ex(
                pair[0] + Vector2::new(2.0, 3.0),
                pair[1] + Vector2::new(2.0, 3.0),
                56.0,
                SHADOW,
            );
            d.draw_line_ex(pair[0], pair[1], 52.0, PAPER);
            d.draw_line_ex(pair[0], pair[1], 40.0, Color::new(175, 180, 165, 255));
        }
        for p in &path {
            d.draw_circle_v(*p, 26.0, PAPER);
            d.draw_circle_v(*p, 20.0, Color::new(175, 180, 165, 255));
        }
        for i in 0..50 {
            let p = route_point(i as f32 / 50.0);
            d.draw_circle_v(l.point(p.0, p.1), 2.5, PAPER);
        }
        // A connecting street and zebra crossing make the route read as a town.
        let cross = l.point(0.36, 0.34);
        d.draw_line_ex(
            l.point(0.04, 0.48),
            l.point(0.35, 0.48),
            25.0,
            Color::new(194, 198, 179, 255),
        );
        for i in 0..5 {
            d.draw_rectangle_rec(
                Rectangle::new(cross.x - 16.0, cross.y - 15.0 + i as f32 * 7.0, 32.0, 4.0),
                PAPER,
            );
        }
        for i in 0..12 {
            let mut p = l.hole(i);
            if run.feedback>0. && run.mistake && run.last_hit==Some(i) {p.y-=(run.feedback*22.).sin().abs()*7.;}
            if run.started && run.result.is_none() && i==s.target && !s.closed[i] {
                let radius=22.*(1.+(1.-s.approach/s.window())*1.5).max(0.65);
                d.draw_ring(p,radius,radius+3.,0.,360.,48,GOLD);
                d.draw_ring(p,21.,23.,0.,360.,48,PAPER);
            }
            let open = s.open[i];
            d.draw_ellipse(p.x as i32, (p.y + 3.0) as i32, 24.0, 17.0, SHADOW);
            d.draw_ellipse(p.x as i32, p.y as i32, 22.0, 16.0, BROWN);
            d.draw_ellipse(
                p.x as i32,
                p.y as i32,
                18.0,
                12.0,
                if open { INK } else { BLUE },
            );
            if open {
                d.draw_ellipse((p.x + 23.0) as i32, (p.y - 14.0) as i32, 18.0, 12.0, BLUE);
                d.draw_line_ex(
                    p + Vector2::new(10.0, -14.0),
                    p + Vector2::new(35.0, -14.0),
                    2.0,
                    PAPER,
                );
                self.text(d, "!", p - Vector2::new(4.0, 15.0), 22.0, GOLD);
                if run.interactive() && (mouse - p).length() < 27.0 {
                    d.draw_ring(p, 25.0, 28.0, 0.0, 360.0, 24, GOLD);
                }
            } else {
                for j in -1..=1 {
                    d.draw_line_ex(
                        p + Vector2::new(-11.0, j as f32 * 4.0),
                        p + Vector2::new(11.0, j as f32 * 4.0),
                        1.0,
                        PAPER,
                    );
                }
                if s.closed[i] {
                    d.draw_line_ex(
                        p + Vector2::new(-6.0, -1.0),
                        p + Vector2::new(-1.0, 4.0),
                        3.0,
                        GREEN,
                    );
                    d.draw_line_ex(
                        p + Vector2::new(-1.0, 4.0),
                        p + Vector2::new(8.0, -5.0),
                        3.0,
                        GREEN,
                    );
                }
            }
        }
        let end = l.point(0.93, 0.80);
        d.draw_line_ex(end, end + Vector2::new(0.0, -65.0), 3.0, INK);
        d.draw_triangle(
            end + Vector2::new(0.0, -65.0),
            end + Vector2::new(0.0, -40.0),
            end + Vector2::new(35.0, -53.0),
            RED,
        );
        self.text(
            d,
            &tr(es, "HOME", "CASA"),
            end + Vector2::new(-20.0, 28.0),
            18.0,
            INK,
        );
        let p = route_point(s.position);
        let mut feet = l.point(p.0, p.1);
        feet.y += if run.started && run.result.is_none() {
            (run.animation * 9.0).sin() * 2.0
        } else {
            0.0
        };
        let fall = if run.result == Some(false) {
            1.0 - (run.result_time * 1.5).min(0.98)
        } else {
            1.0
        };
        self.kenny(
            d,
            feet,
            b.height * 0.14 * fall,
            if run.started {
                (run.animation * 7.0).sin() * 3.0
            } else {
                0.0
            },
            Color::WHITE,
        );
        let label = Rectangle::new(
            b.x + b.width * 0.39,
            b.y + b.height * 0.91,
            b.width * 0.31,
            36.0,
        );
        paper(d, label, PAPER, BROWN);
        self.heading(
            d,
            if run.feedback>0. {run.rating.map_or("South Park",|r|r.label(es))}else{"South Park"},
            Vector2::new(label.x + 13.0, label.y + 2.0),
            22.0,
            INK,
        );
    }
    fn heaters(
        &self,
        d: &mut impl RaylibDraw,
        l: &Layout,
        run: &PuzzleRun,
        h: &Heaters,
        mouse: Vector2,
        es: bool,
    ) {
        let b = l.scene;
        d.draw_rectangle_rec(b, Color::new(204, 222, 228, 255));
        winter(d, l, false);
        self.heading(
            d,
            &tr(es, "South Park heating works", "Calefacción de South Park"),
            l.point(0.06, 0.04),
            25.0,
            INK,
        );
        let first = l.cell(0);
        let last = l.cell(15);
        paper(
            d,
            Rectangle::new(
                first.x - 13.0,
                first.y - 14.0,
                last.x + last.width - first.x + 26.0,
                last.y + last.height - first.y + 28.0,
            ),
            Color::new(108, 142, 155, 255),
            INK,
        );
        for (x, y) in [
            (first.x - 7.0, first.y - 7.0),
            (last.x + last.width + 7.0, first.y - 7.0),
            (first.x - 7.0, last.y + last.height + 7.0),
            (last.x + last.width + 7.0, last.y + last.height + 7.0),
        ] {
            d.draw_circle(x as i32, y as i32, 3.0, INK);
            d.draw_line_ex(
                Vector2::new(x - 1.0, y - 1.0),
                Vector2::new(x + 1.0, y + 1.0),
                1.0,
                PAPER,
            );
        }
        for i in 0..16 {
            let r = l.cell(i);
            let c = Vector2::new(r.x + r.width * 0.5, r.y + r.height * 0.5);
            let hover = run.interactive() && r.check_collision_point_rec(mouse);
            d.draw_rectangle_rounded(
                r,
                0.12,
                6,
                if hover {
                    PAPER
                } else {
                    Color::new(227, 235, 229, 255)
                },
            );
            let color = if h.powered[i] { GOLD } else { BLUE };
            for (bit, dx, dy) in [(1, 0.0, -0.5), (2, 0.5, 0.0), (4, 0.0, 0.5), (8, -0.5, 0.0)] {
                if h.tiles[i] & bit != 0 {
                    let end = c + Vector2::new(dx * (r.width + 3.0), dy * (r.height + 3.0));
                    d.draw_line_ex(
                        c + Vector2::new(1.0, 2.0),
                        end + Vector2::new(1.0, 2.0),
                        12.0,
                        INK,
                    );
                    d.draw_line_ex(c, end, 8.0, color);
                    d.draw_line_ex(
                        c + Vector2::new(-1.0, -1.0),
                        end + Vector2::new(-1.0, -1.0),
                        2.0,
                        PAPER,
                    );
                }
            }
            d.draw_circle_v(c, 9.0, color);
            d.draw_circle_v(c, 3.0, INK);
            if run.feedback > 0.0 && run.last_hit == Some(i) {
                d.draw_ring(c, r.width * 0.32, r.width * 0.35, 0.0, 300.0, 24, GOLD);
            }
        }
        let source = Vector2::new(first.x - 33.0, first.y + first.height * 0.5);
        d.draw_line_ex(source, Vector2::new(first.x, source.y), 8.0, GOLD);
        d.draw_circle_v(source, 18.0, INK);
        d.draw_circle_v(source, 14.0, GOLD);
        self.text(d, "+", source - Vector2::new(7.0, 17.0), 26.0, INK);
        for (n, i) in [3, 11, 15].into_iter().enumerate() {
            let r = l.cell(i);
            let x = r.x + r.width + 38.0;
            let y = r.y + r.height * 0.5;
            d.draw_line_ex(
                Vector2::new(r.x + r.width, y),
                Vector2::new(x, y),
                7.0,
                if h.destinations[n] { GOLD } else { INK },
            );
            if h.destinations[n] {
                d.draw_circle(x as i32, y as i32, 43.0, Color::new(233, 191, 98, 55));
            }
            paper(
                d,
                Rectangle::new(x - 17.0, y - 27.0, 41.0, 54.0),
                if h.destinations[n] { GOLD } else { PAPER },
                INK,
            );
            for k in 0..4 {
                d.draw_line_ex(
                    Vector2::new(x - 9.0 + k as f32 * 8.0, y - 17.0),
                    Vector2::new(x - 9.0 + k as f32 * 8.0, y + 17.0),
                    4.0,
                    if h.destinations[n] { RED } else { BLUE },
                );
            }
            self.text(
                d,
                &(n + 1).to_string(),
                Vector2::new(x - 2.0, y + 29.0),
                18.0,
                INK,
            );
        }
        let cold = 1.0 - h.destinations.iter().filter(|x| **x).count() as f32 / 3.0;
        let feet =
            l.point(0.86, 0.82) + Vector2::new((run.animation * 28.0).sin() * cold * 1.8, 0.0);
        self.kenny(
            d,
            feet,
            b.height * 0.29,
            (run.animation * 18.0).sin() * cold * 1.6,
            if run.result == Some(false) {
                Color::new(135, 206, 235, 255)
            } else {
                Color::WHITE
            },
        );
        if run.result == Some(false) {
            paper(
                d,
                Rectangle::new(
                    feet.x - 49.0,
                    feet.y - b.height * 0.29,
                    98.0,
                    b.height * 0.29,
                ),
                Color::new(170, 226, 239, 130),
                BLUE,
            );
        }
        self.text(
            d,
            &tr(
                es,
                "3 heaters. One very cold kid.",
                "3 calefactores. Un niño congelado.",
            ),
            l.point(0.07, 0.92),
            20.0,
            INK,
        );
    }
}

fn paper(d: &mut impl RaylibDraw, r: Rectangle, color: Color, edge: Color) {
    d.draw_rectangle_rounded(
        Rectangle::new(r.x + 2.0, r.y + 4.0, r.width, r.height),
        0.08,
        8,
        SHADOW,
    );
    d.draw_rectangle_rounded(r, 0.08, 8, color);
    d.draw_rectangle_rounded_lines_ex(r, 0.08, 8, 1.5, edge);
}
fn meter(d: &mut impl RaylibDraw, r: Rectangle, value: f32, color: Color) {
    d.draw_rectangle_rounded(r, 0.8, 8, Color::new(218, 208, 187, 255));
    if value > 0.0 {
        d.draw_rectangle_rounded(
            Rectangle::new(r.x, r.y, r.width * value.clamp(0.0, 1.0), r.height),
            0.8,
            8,
            color,
        );
    }
}
fn portrait(d: &mut impl RaylibDraw, t: &Texture2D, r: Rectangle, roll: f32, tint: Color) {
    let scale = (r.width / t.width as f32).min(r.height / t.height as f32);
    let (w, h) = (t.width as f32 * scale, t.height as f32 * scale);
    let dest = Rectangle::new(r.x + r.width * 0.5, r.y + r.height, w, h);
    d.draw_texture_pro(
        t,
        Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32),
        Rectangle::new(dest.x + 2.0, dest.y + 3.0, w, h),
        Vector2::new(w * 0.5, h),
        roll,
        SHADOW,
    );
    d.draw_texture_pro(
        t,
        Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32),
        dest,
        Vector2::new(w * 0.5, h),
        roll,
        tint,
    );
}
fn tree(d: &mut impl RaylibDraw, p: Vector2, size: f32) {
    d.draw_rectangle(
        (p.x - 3.0) as i32,
        (p.y - size * 0.32) as i32,
        6,
        (size * 0.32) as i32,
        BROWN,
    );
    for i in 0..3 {
        let y = p.y - size + i as f32 * size * 0.20;
        let w = size * (0.25 + i as f32 * 0.035);
        d.draw_triangle(
            Vector2::new(p.x, y),
            Vector2::new(p.x - w, y + size * 0.48),
            Vector2::new(p.x + w, y + size * 0.48),
            if i % 2 == 0 {
                GREEN
            } else {
                Color::new(92, 130, 110, 255)
            },
        );
    }
}
fn house(d: &mut impl RaylibDraw, p: Vector2, size: f32, color: Color) {
    d.draw_rectangle_rec(
        Rectangle::new(
            p.x + size * 0.73,
            p.y - size * 0.36,
            size * 0.12,
            size * 0.28,
        ),
        BROWN,
    );
    d.draw_rectangle_rec(
        Rectangle::new(p.x + 3.0, p.y + 4.0, size, size * 0.65),
        SHADOW,
    );
    d.draw_rectangle_rec(Rectangle::new(p.x, p.y, size, size * 0.65), color);
    d.draw_rectangle_rec(
        Rectangle::new(p.x + size * 0.88, p.y, size * 0.12, size * 0.65),
        Color::new(73, 56, 45, 30),
    );
    d.draw_triangle(
        Vector2::new(p.x + size * 0.5, p.y - size * 0.34),
        Vector2::new(p.x - 5.0, p.y + 4.0),
        Vector2::new(p.x + size + 5.0, p.y + 4.0),
        BROWN,
    );
    d.draw_line_ex(
        Vector2::new(p.x - 4.0, p.y + 2.0),
        Vector2::new(p.x + size * 0.5, p.y - size * 0.34),
        4.0,
        PAPER,
    );
    d.draw_rectangle_rec(
        Rectangle::new(
            p.x + size * 0.58,
            p.y + size * 0.24,
            size * 0.22,
            size * 0.41,
        ),
        INK,
    );
    d.draw_rectangle_rec(
        Rectangle::new(
            p.x + size * 0.13,
            p.y + size * 0.14,
            size * 0.26,
            size * 0.26,
        ),
        PAPER,
    );
    d.draw_rectangle_rec(
        Rectangle::new(
            p.x + size * 0.16,
            p.y + size * 0.17,
            size * 0.20,
            size * 0.20,
        ),
        BLUE,
    );
    d.draw_line_ex(
        Vector2::new(p.x + size * 0.26, p.y + size * 0.17),
        Vector2::new(p.x + size * 0.26, p.y + size * 0.37),
        2.0,
        PAPER,
    );
    d.draw_line_ex(
        Vector2::new(p.x + size * 0.10, p.y + size * 0.42),
        Vector2::new(p.x + size * 0.41, p.y + size * 0.42),
        3.0,
        BROWN,
    );
    d.draw_rectangle_rec(
        Rectangle::new(p.x + size * 0.55, p.y + size * 0.64, size * 0.29, 3.0),
        PAPER,
    );
    d.draw_circle(
        (p.x + size * 0.74) as i32,
        (p.y + size * 0.46) as i32,
        1.5,
        GOLD,
    );
}
fn winter(d: &mut impl RaylibDraw, l: &Layout, night: bool) {
    let b = l.scene;
    for i in 0..6 {
        let x = b.x + i as f32 * b.width / 5.0;
        d.draw_triangle(
            Vector2::new(x, b.y + b.height * 0.08),
            Vector2::new(x - b.width * 0.22, b.y + b.height * 0.57),
            Vector2::new(x + b.width * 0.25, b.y + b.height * 0.57),
            if night {
                Color::new(69, 74, 104, 255)
            } else {
                Color::new(181, 203, 210, 255)
            },
        );
    }
    d.draw_ellipse(
        (b.x + b.width * 0.5) as i32,
        (b.y + b.height) as i32,
        b.width * 0.8,
        b.height * 0.30,
        if night {
            Color::new(114, 142, 159, 255)
        } else {
            PAPER
        },
    );
    for i in 0..6 {
        tree(
            d,
            l.point(0.04 + i as f32 * 0.185, 0.86),
            b.height * (0.14 + (i % 2) as f32 * 0.04),
        );
    }
}
fn school_bus(d: &mut impl RaylibDraw, p: Vector2, width: f32) {
    let height = width * 0.40;
    paper(
        d,
        Rectangle::new(p.x, p.y - height, width, height),
        GOLD,
        INK,
    );
    for i in 0..5 {
        d.draw_rectangle_rec(
            Rectangle::new(
                p.x + width * 0.07 + i as f32 * width * 0.15,
                p.y - height * 0.85,
                width * 0.12,
                height * 0.42,
            ),
            BLUE,
        );
    }
    d.draw_rectangle_rec(
        Rectangle::new(
            p.x + width * 0.84,
            p.y - height * 0.78,
            width * 0.12,
            height * 0.7,
        ),
        BROWN,
    );
    d.draw_line_ex(
        Vector2::new(p.x + 4.0, p.y - height * 0.26),
        Vector2::new(p.x + width - 4.0, p.y - height * 0.26),
        3.0,
        INK,
    );
    for x in [0.19, 0.80] {
        let centre = Vector2::new(p.x + width * x, p.y);
        d.draw_circle_v(centre, height * 0.21, INK);
        d.draw_circle_v(centre, height * 0.10, BLUE);
        d.draw_circle_v(centre, height * 0.04, PAPER);
    }
}
fn accent(k: ThreatKind) -> Color {
    match k {
        ThreatKind::Snowman => BLUE,
        ThreatKind::Ufo => Color::new(145, 123, 174, 255),
        ThreatKind::Bus => GOLD,
        ThreatKind::Piano => RED,
        ThreatKind::Sewer => GREEN,
    }
}
fn title(k: ThreatKind, es: bool) -> String {
    match k {
        ThreatKind::Snowman => tr(es, "Cold feet", "Frío de muerte"),
        ThreatKind::Ufo => tr(es, "Lights out!", "¡Fuera luces!"),
        ThreatKind::Bus => tr(es, "Who Will Hit Kenny?", "¿Quién atropellará a Kenny?"),
        ThreatKind::Piano => tr(es, "A minor disaster", "Un desastre menor"),
        ThreatKind::Sewer => tr(es, "Watch your step", "Mira por dónde vas"),
    }
}
fn instructions(k: ThreatKind, es: bool) -> String {
    match k {
    ThreatKind::Snowman=>tr(es,"Click to rotate each cable. Connect the + source to all 3 heaters before Kenny freezes.\nYou have 30 seconds.","Gira los cables con un clic. Conecta la fuente + a los 3 calefactores antes de que Kenny se congele.\nTienes 30 segundos."),
    ThreatKind::Ufo=>tr(es,"Press A, S or D as the shrinking ring meets its circle.\n20 targets. PERFECT: 100 points; GOOD: 60. Keep the beam below 100%.","Pulsa A, S o D cuando el anillo coincida con su círculo.\n20 objetivos. PERFECTO: 100 puntos; BIEN: 60. Evita que el rayo llegue al 100%."),
    ThreatKind::Bus=>tr(es,"One final token per row and column. Click: candidate. Hold 0.45 s: fix. X: discard. Buildings, kiosks and planter block cells. Confirm once.","Una ficha fija por fila y columna. Clic: candidato. Mantén 0,45 s: fijar. X: descarte. Edificios, quioscos y jardinera bloquean. Confirma una vez."),
    ThreatKind::Piano=>tr(es,"Watch and listen, then repeat the notes. Eight rounds at a steady tempo.\nOne mistake per round is allowed. A second drops the piano.","Observa y escucha, luego repite las notas. Ocho rondas al mismo ritmo.\nPuedes fallar una vez por ronda. Al segundo error cae el piano."),
    ThreatKind::Sewer=>tr(es,"Click each lid when its shrinking ring meets the rim.\n12 lids, three faster stages. Early clicks bounce open. Save Kenny before he steps in!","Pulsa cada tapa cuando el anillo coincida con su borde.\n12 tapas, tres tramos cada vez más rápidos. Si te adelantas, rebota y se abre."),
}
}
fn short_instructions(k: ThreatKind, es: bool) -> String {
    match k {
        ThreatKind::Snowman => tr(
            es,
            "Click to turn cables. Connect + to 3 heaters.\n30 seconds!",
            "Gira los cables. Conecta + a 3 calefactores.\n¡30 segundos!",
        ),
        ThreatKind::Ufo => tr(
            es,
            "A / S / D when rings meet.\n20 targets. Keep beam below 100%.",
            "A / S / D al coincidir los anillos.\n20 objetivos. Rayo por debajo del 100%.",
        ),
        ThreatKind::Bus => tr(es,"One final token per row/column. Click: note. Hold 0.45 s: fix. X: discard. Buildings, kiosks, planter: blocked.","Una ficha fija por fila/columna. Clic: nota. Mantén 0,45 s: fijar. X: descarte. Edificios, quioscos y jardinera: bloqueados."),
        ThreatKind::Piano => tr(
            es,
            "Watch, listen, repeat.\n8 rounds, same tempo.\nSecond error in a round: defeat.",
            "Observa, escucha y repite.\n8 rondas, mismo ritmo.\nSegundo error por ronda: derrota.",
        ),
        ThreatKind::Sewer => tr(
            es,
            "Click when ring meets lid.\n12 lids, 3 stages.\nEarly clicks bounce open!",
            "Pulsa al coincidir anillo y tapa.\n12 tapas, 3 tramos.\n¡Si te adelantas, se abre!",
        ),
    }
}
fn aside(k: ThreatKind, es: bool) -> String {
    match k {
        ThreatKind::Snowman => tr(
            es,
            "“Breathe, Tweek. Just breathe.”",
            "«Respira, Tweek. Respira.»",
        ),
        ThreatKind::Ufo => tr(
            es,
            "“Aliens. Of course it's aliens.”",
            "«Extraterrestres. Cómo no.»",
        ),
        ThreatKind::Bus => tr(es, "“Me? Nah. You do it.”", "«¿Yo? Nah. Hazlo tú.»"),
        ThreatKind::Piano => tr(
            es,
            "“A-a-and now, the encore!”",
            "«¡Y a-a-ahora, otra más!»",
        ),
        ThreatKind::Sewer => tr(
            es,
            "“This is about Stan, isn't it?”",
            "«Esto es por Stan, ¿verdad?»",
        ),
    }
}
fn status(r: &PuzzleRun, es: bool) -> String {
    match &r.board {
        Board::Heaters(h) => format!(
            "{} / 3    •    {:.0} s",
            h.destinations.iter().filter(|x| **x).count(),
            (30.0 - r.elapsed).max(0.0)
        ),
        Board::Ufo(u) => format!("{} {:.0}% | {}/20\n{}: {}",tr(es,"Beam","Rayo"),u.charge*100.,u.completed,tr(es,"Score","Puntos"),u.score),
        Board::Bus(_) => tr(es,"No time limit.","Sin límite de tiempo."),
        Board::Piano(p) => format!("{} {}/8\n{}: {} | {}/{}",tr(es,"Round","Ronda"),p.round+1,tr(es,"Errors left","Fallos libres"),1_u8.saturating_sub(p.errors),p.input,p.length()),
        Board::Sewers(s) => format!(
            "{}: {} / 12",
            tr(es, "Lids closed", "Tapas cerradas"),
            s.closed.iter().filter(|x| **x).count()
        ),
    }
}
fn progress(r: &PuzzleRun) -> f32 {
    match &r.board {
        Board::Heaters(_) => (30.0 - r.elapsed) / 30.0,
        Board::Ufo(u) => u.charge,
        Board::Bus(_) => 0.0,
        Board::Piano(p) => (p.round as f32) / 8.0,
        Board::Sewers(s) => s.position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn illustrated_controls_and_hit_regions_agree_at_all_supported_sizes() {
        for (w, h) in [
            (640, 480),
            (800, 600),
            (1280, 800),
            (1920, 1080),
            (800, 1000),
        ] {
            let l = Layout::new(w, h);
            for r in [l.audio(), l.pause(), l.primary(), l.restart(), l.back()] {
                assert!(
                    r.x >= 0.0
                        && r.y >= 0.0
                        && r.x + r.width <= l.width
                        && r.y + r.height <= l.height
                );
                let p = Vector2::new(r.x + r.width * 0.5, r.y + r.height * 0.5);
                assert!((l.mouse(p * l.scale + l.offset) - p).length() < 0.001);
            }
            for kind in ThreatKind::ALL {
                let p = PuzzleRun::new(kind, 0);
                let count = match kind {
                    ThreatKind::Snowman => 16,
                    ThreatKind::Ufo => 3,
                    ThreatKind::Bus => 36,
                    ThreatKind::Piano => 5,
                    ThreatKind::Sewer => 12,
                };
                for i in 0..count {
                    let point = match kind {
                        ThreatKind::Ufo => l.light(i),
                        ThreatKind::Sewer => l.hole(i),
                        _ => {
                            let r = match kind {
                                ThreatKind::Snowman => l.cell(i),
                                ThreatKind::Bus => l.case_cell(i),
                                _ => l.key(i, 0),
                            };
                            Vector2::new(r.x + r.width * 0.5, r.y + r.height * 0.5)
                        }
                    };
                    assert!(l.scene.check_collision_point_rec(point));
                    assert_eq!(l.hit(point, &p), Some(i));
                }
            }
        }
    }
}
