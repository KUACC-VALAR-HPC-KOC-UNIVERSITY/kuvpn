use iced::Color;
use serde::{Deserialize, Serialize};

/// All semantic color slots for a palette variant. All fields are `Color` which
/// is `Copy`, so `Palette` itself is `Copy` — safe to capture in `'static` closures.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub border: Color,
    pub border_subtle: Color,

    pub accent: Color,
    pub accent_hover: Color,

    pub success: Color,
    pub warning: Color,
    pub danger: Color,

    pub text: Color,
    pub text_muted: Color,
    pub text_disabled: Color,

    pub overlay: Color,
}

/// Named color families — ten aesthetic moods, each with a dark and light variant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum PaletteFamily {
    /// KU Burgundy — the current default palette.
    #[default]
    Crimson,
    /// Cool steel-blue (Breeze / KDE-inspired).
    Frost,
    /// Neutral gray (Adwaita / GNOME-inspired).
    Pebble,
    /// Blue-gray (Windows Fluent-inspired).
    Slate,
    /// Warm gray (macOS-inspired).
    Sand,
    /// Dark green with vivid emerald accent.
    Forest,
    /// Dark teal with bright cyan accent.
    Ocean,
    /// Dark purple with vivid violet accent.
    Violet,
    /// Dark amber with bright warm accent.
    Ember,
    /// Dark pink with vivid rose accent.
    Rose,
}

impl PaletteFamily {
    pub const ALL: [PaletteFamily; 10] = [
        PaletteFamily::Crimson,
        PaletteFamily::Frost,
        PaletteFamily::Pebble,
        PaletteFamily::Slate,
        PaletteFamily::Sand,
        PaletteFamily::Forest,
        PaletteFamily::Ocean,
        PaletteFamily::Violet,
        PaletteFamily::Ember,
        PaletteFamily::Rose,
    ];

    pub fn palette(self, dark: bool) -> Palette {
        match (self, dark) {
            (PaletteFamily::Crimson, true) => crimson_dark(),
            (PaletteFamily::Crimson, false) => crimson_light(),
            (PaletteFamily::Frost, true) => frost_dark(),
            (PaletteFamily::Frost, false) => frost_light(),
            (PaletteFamily::Pebble, true) => pebble_dark(),
            (PaletteFamily::Pebble, false) => pebble_light(),
            (PaletteFamily::Slate, true) => slate_dark(),
            (PaletteFamily::Slate, false) => slate_light(),
            (PaletteFamily::Sand, true) => sand_dark(),
            (PaletteFamily::Sand, false) => sand_light(),
            (PaletteFamily::Forest, true) => forest_dark(),
            (PaletteFamily::Forest, false) => forest_light(),
            (PaletteFamily::Ocean, true) => ocean_dark(),
            (PaletteFamily::Ocean, false) => ocean_light(),
            (PaletteFamily::Violet, true) => violet_dark(),
            (PaletteFamily::Violet, false) => violet_light(),
            (PaletteFamily::Ember, true) => ember_dark(),
            (PaletteFamily::Ember, false) => ember_light(),
            (PaletteFamily::Rose, true) => rose_dark(),
            (PaletteFamily::Rose, false) => rose_light(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PaletteFamily::Crimson => "Crimson",
            PaletteFamily::Frost => "Frost",
            PaletteFamily::Pebble => "Pebble",
            PaletteFamily::Slate => "Slate",
            PaletteFamily::Sand => "Sand",
            PaletteFamily::Forest => "Forest",
            PaletteFamily::Ocean => "Ocean",
            PaletteFamily::Violet => "Violet",
            PaletteFamily::Ember => "Ember",
            PaletteFamily::Rose => "Rose",
        }
    }
}

// ── Palette definitions ───────────────────────────────────────────────────────
fn crimson_dark() -> Palette {
    Palette {
        bg: iced::color!(0x130F10),
        surface: iced::color!(0x201B1D),
        surface_raised: iced::color!(0x2A2326),
        border: iced::color!(0x433B3E),
        border_subtle: iced::color!(0x312B2D),
        accent: iced::color!(0xA0213C),
        accent_hover: iced::color!(0xBD2D49),
        // warm vivid green + golden amber — warm tones echo the burgundy without clashing
        success: iced::color!(0x3EA64F),
        warning: iced::color!(0xCC8014),
        danger: iced::color!(0xD63B3B),
        text: iced::color!(0xDAD8D9),
        text_muted: iced::color!(0x837E80),
        text_disabled: iced::color!(0x585456),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn crimson_light() -> Palette {
    Palette {
        bg: iced::color!(0xFAF4F6),
        surface: iced::color!(0xF0E7EB),
        surface_raised: iced::color!(0xE3D8DC),
        border: iced::color!(0xC4AAB4),
        border_subtle: iced::color!(0xD9CDD2),
        accent: iced::color!(0x8C1C30),
        accent_hover: iced::color!(0x6E1223),
        success: iced::color!(0x1A8C3D),
        warning: iced::color!(0xBF7A0A),
        danger: iced::color!(0xB51C1C),
        text: iced::color!(0x1A1214),
        text_muted: iced::color!(0x6B5C62),
        text_disabled: iced::color!(0x9E9094),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn frost_dark() -> Palette {
    Palette {
        bg: iced::color!(0x0F141A),
        surface: iced::color!(0x1A212B),
        surface_raised: iced::color!(0x212B38),
        border: iced::color!(0x384C61),
        border_subtle: iced::color!(0x293847),
        accent: iced::color!(0x2E7AB8),
        accent_hover: iced::color!(0x3D8CD1),
        // cool icy mint green + cool chartreuse-yellow — icy like the steel-blue accent
        success: iced::color!(0x40BF80),
        warning: iced::color!(0xB8C72E),
        danger: iced::color!(0xCC3333),
        text: iced::color!(0xE0E6EB),
        text_muted: iced::color!(0x8094A6),
        text_disabled: iced::color!(0x596673),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn frost_light() -> Palette {
    Palette {
        bg: iced::color!(0xF5F7FC),
        surface: iced::color!(0xEBF0F7),
        surface_raised: iced::color!(0xDEE6F0),
        border: iced::color!(0xB2C4DE),
        border_subtle: iced::color!(0xD1DEED),
        accent: iced::color!(0x2E7AB8),
        accent_hover: iced::color!(0x1F6199),
        success: iced::color!(0x1F8557),
        warning: iced::color!(0xCC9A08),
        danger: iced::color!(0xB81F1F),
        text: iced::color!(0x141F2E),
        text_muted: iced::color!(0x5C758F),
        text_disabled: iced::color!(0x94A6B8),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn pebble_dark() -> Palette {
    Palette {
        bg: iced::color!(0x141414),
        surface: iced::color!(0x212121),
        surface_raised: iced::color!(0x2B2B2B),
        border: iced::color!(0x474747),
        border_subtle: iced::color!(0x333333),
        accent: iced::color!(0x6699D9),
        accent_hover: iced::color!(0x7AADF2),
        // clean vivid green + warm amber — neutral grey palette lets these pop clearly
        success: iced::color!(0x52B866),
        warning: iced::color!(0xD9AD2E),
        danger: iced::color!(0xCC3838),
        text: iced::color!(0xE0E0E0),
        text_muted: iced::color!(0x8C8C8C),
        text_disabled: iced::color!(0x616161),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn pebble_light() -> Palette {
    Palette {
        bg: iced::color!(0xF5F5F5),
        surface: iced::color!(0xEDEDED),
        surface_raised: iced::color!(0xE0E0E0),
        border: iced::color!(0xBABABA),
        border_subtle: iced::color!(0xD4D4D4),
        accent: iced::color!(0x4073BF),
        accent_hover: iced::color!(0x2E599E),
        success: iced::color!(0x1E8C3D),
        warning: iced::color!(0xBF8A0D),
        danger: iced::color!(0xB82626),
        text: iced::color!(0x1A1A1A),
        text_muted: iced::color!(0x6B6B6B),
        text_disabled: iced::color!(0x9E9E9E),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn slate_dark() -> Palette {
    Palette {
        bg: iced::color!(0x12171C),
        surface: iced::color!(0x1C242B),
        surface_raised: iced::color!(0x242E38),
        border: iced::color!(0x405261),
        border_subtle: iced::color!(0x2E3D4C),
        accent: iced::color!(0x407AB8),
        accent_hover: iced::color!(0x528FD1),
        // blue-green teal + golden-steel yellow — both echo the slate-blue character
        success: iced::color!(0x38AD85),
        warning: iced::color!(0xBFB833),
        danger: iced::color!(0xC73838),
        text: iced::color!(0xE0E6EB),
        text_muted: iced::color!(0x8594A6),
        text_disabled: iced::color!(0x616B7A),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn slate_light() -> Palette {
    Palette {
        bg: iced::color!(0xF2F5FA),
        surface: iced::color!(0xE8EDF5),
        surface_raised: iced::color!(0xDBE0EB),
        border: iced::color!(0xB2C2D9),
        border_subtle: iced::color!(0xD1DBEB),
        accent: iced::color!(0x2E6BAD),
        accent_hover: iced::color!(0x1F528C),
        success: iced::color!(0x1A805C),
        warning: iced::color!(0xCC9A08),
        danger: iced::color!(0xB21F1F),
        text: iced::color!(0x141A24),
        text_muted: iced::color!(0x5C6E85),
        text_disabled: iced::color!(0x94A3B8),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn sand_dark() -> Palette {
    Palette {
        bg: iced::color!(0x171412),
        surface: iced::color!(0x24211C),
        surface_raised: iced::color!(0x2E2B26),
        border: iced::color!(0x4C453D),
        border_subtle: iced::color!(0x38332E),
        accent: iced::color!(0x8C6B42),
        accent_hover: iced::color!(0xA67D52),
        // olive-warm green + deep golden amber — earthy tones complement warm sand/tan
        success: iced::color!(0x70A33D),
        warning: iced::color!(0xEBB21F),
        danger: iced::color!(0xCC3838),
        text: iced::color!(0xE6E0D9),
        text_muted: iced::color!(0x8C857A),
        text_disabled: iced::color!(0x666159),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn sand_light() -> Palette {
    Palette {
        bg: iced::color!(0xFAF7F0),
        surface: iced::color!(0xF2EDE6),
        surface_raised: iced::color!(0xE6E0D6),
        border: iced::color!(0xC4BAA8),
        border_subtle: iced::color!(0xDED4C7),
        accent: iced::color!(0x7A5933),
        accent_hover: iced::color!(0x614224),
        success: iced::color!(0x3D8C1F),
        warning: iced::color!(0xC28A0A),
        danger: iced::color!(0xB82424),
        text: iced::color!(0x1F1A14),
        text_muted: iced::color!(0x70665C),
        text_disabled: iced::color!(0xA3998C),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn forest_dark() -> Palette {
    Palette {
        bg: iced::color!(0x0D140F),
        surface: iced::color!(0x141F17),
        surface_raised: iced::color!(0x1C2B21),
        border: iced::color!(0x33523D),
        border_subtle: iced::color!(0x243829),
        accent: iced::color!(0x26AD61),
        accent_hover: iced::color!(0x38C775),
        // vivid emerald (near-accent) + bright lime-yellow — forest energy, natural vitality
        success: iced::color!(0x29D166),
        warning: iced::color!(0xCCCC14),
        danger: iced::color!(0xCC3838),
        text: iced::color!(0xD9EBDE),
        text_muted: iced::color!(0x7A9985),
        text_disabled: iced::color!(0x526B5C),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn forest_light() -> Palette {
    Palette {
        bg: iced::color!(0xF2F7F2),
        surface: iced::color!(0xE8F0E8),
        surface_raised: iced::color!(0xDBE6DB),
        border: iced::color!(0xA8C7AD),
        border_subtle: iced::color!(0xCCE0CF),
        accent: iced::color!(0x1A8547),
        accent_hover: iced::color!(0x126633),
        success: iced::color!(0x0F8F38),
        warning: iced::color!(0xCC9E08),
        danger: iced::color!(0xB21F1F),
        text: iced::color!(0x122417),
        text_muted: iced::color!(0x4C7557),
        text_disabled: iced::color!(0x8FAD96),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn ocean_dark() -> Palette {
    Palette {
        bg: iced::color!(0x0A141A),
        surface: iced::color!(0x121F29),
        surface_raised: iced::color!(0x1A2B38),
        border: iced::color!(0x2E5266),
        border_subtle: iced::color!(0x1F384C),
        accent: iced::color!(0x1AADBF),
        accent_hover: iced::color!(0x2EC7DB),
        // oceanic teal-green + golden-aqua yellow — warm gold contrasts the cool cyan
        success: iced::color!(0x2EC294),
        warning: iced::color!(0xADCC24),
        danger: iced::color!(0xCC3847),
        text: iced::color!(0xD4EDF5),
        text_muted: iced::color!(0x6B9EB2),
        text_disabled: iced::color!(0x476B80),
        overlay: iced::color!(0x000000, 0.85),
    }
}

fn ocean_light() -> Palette {
    Palette {
        bg: iced::color!(0xF0F7FA),
        surface: iced::color!(0xE8F0F7),
        surface_raised: iced::color!(0xD9E6ED),
        border: iced::color!(0xA3C7DB),
        border_subtle: iced::color!(0xC9E0ED),
        accent: iced::color!(0x0D8599),
        accent_hover: iced::color!(0x08667A),
        success: iced::color!(0x0F8566),
        warning: iced::color!(0xCC9A0A),
        danger: iced::color!(0xAD1F29),
        text: iced::color!(0x0D1F2E),
        text_muted: iced::color!(0x42758F),
        text_disabled: iced::color!(0x87ADC2),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn violet_dark() -> Palette {
    Palette {
        bg: iced::color!(0x120D1A),
        surface: iced::color!(0x1C1429),
        surface_raised: iced::color!(0x261C38),
        border: iced::color!(0x473366),
        border_subtle: iced::color!(0x332447),
        accent: iced::color!(0xA659F2),
        accent_hover: iced::color!(0xB870FF),
        // vivid electric green + vivid gold — both pop sharply against deep purple
        success: iced::color!(0x47D973),
        warning: iced::color!(0xEBC714),
        danger: iced::color!(0xCC384C),
        text: iced::color!(0xE6DBF5),
        text_muted: iced::color!(0x8C73B2),
        text_disabled: iced::color!(0x614C80),
        overlay: iced::color!(0x000000, 0.88),
    }
}

fn violet_light() -> Palette {
    Palette {
        bg: iced::color!(0xF7F5FC),
        surface: iced::color!(0xF0EBF7),
        surface_raised: iced::color!(0xE3DEF0),
        border: iced::color!(0xC2B2E0),
        border_subtle: iced::color!(0xDED4F0),
        accent: iced::color!(0x8538CC),
        accent_hover: iced::color!(0x6626A6),
        success: iced::color!(0x1F9442),
        warning: iced::color!(0xC29208),
        danger: iced::color!(0xB21F2E),
        text: iced::color!(0x1A0F2E),
        text_muted: iced::color!(0x664C8F),
        text_disabled: iced::color!(0xA18FBA),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn ember_dark() -> Palette {
    Palette {
        bg: iced::color!(0x17120A),
        surface: iced::color!(0x241C0F),
        surface_raised: iced::color!(0x302614),
        border: iced::color!(0x59421F),
        border_subtle: iced::color!(0x3D2E14),
        accent: iced::color!(0xEB8C14),
        accent_hover: iced::color!(0xFFA326),
        // earthy warm green + vivid lime — green contrasts the amber/orange character
        success: iced::color!(0x57A642),
        warning: iced::color!(0xD9E014),
        danger: iced::color!(0xD1382E),
        text: iced::color!(0xF5E6CC),
        text_muted: iced::color!(0x998057),
        text_disabled: iced::color!(0x6B5738),
        overlay: iced::color!(0x000000, 0.88),
    }
}

fn ember_light() -> Palette {
    Palette {
        bg: iced::color!(0xFCF7ED),
        surface: iced::color!(0xF5EDE0),
        surface_raised: iced::color!(0xE8E0D1),
        border: iced::color!(0xCFBA9C),
        border_subtle: iced::color!(0xE6D6BA),
        accent: iced::color!(0xB86105),
        accent_hover: iced::color!(0x944703),
        success: iced::color!(0x2E8C24),
        warning: iced::color!(0xBD8508),
        danger: iced::color!(0xB2241A),
        text: iced::color!(0x261A0A),
        text_muted: iced::color!(0x755729),
        text_disabled: iced::color!(0xA88F66),
        overlay: iced::color!(0x000000, 0.60),
    }
}

fn rose_dark() -> Palette {
    Palette {
        bg: iced::color!(0x170D12),
        surface: iced::color!(0x24141C),
        surface_raised: iced::color!(0x2E1C26),
        border: iced::color!(0x592E42),
        border_subtle: iced::color!(0x3D1F2E),
        accent: iced::color!(0xE64C8C),
        accent_hover: iced::color!(0xFF66A3),
        // fresh vivid green + warm peachy-amber — both contrast the rose/pink character
        success: iced::color!(0x42C26B),
        warning: iced::color!(0xE6B226),
        danger: iced::color!(0xD12E38),
        text: iced::color!(0xF5E0EB),
        text_muted: iced::color!(0x996685),
        text_disabled: iced::color!(0x6B4257),
        overlay: iced::color!(0x000000, 0.88),
    }
}

fn rose_light() -> Palette {
    Palette {
        bg: iced::color!(0xFCF5FA),
        surface: iced::color!(0xF5EBF2),
        surface_raised: iced::color!(0xE8DEE6),
        border: iced::color!(0xCFB2C7),
        border_subtle: iced::color!(0xE6D1E0),
        accent: iced::color!(0xBF2E66),
        accent_hover: iced::color!(0x991F4C),
        success: iced::color!(0x1F8542),
        warning: iced::color!(0xC28A12),
        danger: iced::color!(0xB21A24),
        text: iced::color!(0x290F1A),
        text_muted: iced::color!(0x75425C),
        text_disabled: iced::color!(0xA88594),
        overlay: iced::color!(0x000000, 0.60),
    }
}

impl std::fmt::Display for PaletteFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ── Rounding ──────────────────────────────────────────────────────────────────

/// Corner radius style. `Copy` — safe to move into `'static` closures.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Rounding {
    /// 0 px — completely square corners.
    Square,
    /// 4 px — crisp, nearly square (Windows-flavoured).
    Sharp,
    /// 10 px — clearly rounded (Breeze / KDE).
    #[default]
    Rounded,
    /// 16 px — soft, card-like (Adwaita / GNOME).
    Smooth,
    /// 24 px — pill-shaped buttons, large cards.
    Pill,
}

impl Rounding {
    /// Main corner radius for cards and buttons.
    pub fn radius(self) -> f32 {
        match self {
            Rounding::Square => 0.0,
            Rounding::Sharp => 4.0,
            Rounding::Rounded => 10.0,
            Rounding::Smooth => 16.0,
            Rounding::Pill => 24.0,
        }
    }

    /// Smaller radius for inputs, badges, segment buttons.
    pub fn small_radius(self) -> f32 {
        match self {
            Rounding::Square => 0.0,
            Rounding::Sharp => 2.0,
            Rounding::Rounded => 6.0,
            Rounding::Smooth => 10.0,
            Rounding::Pill => 14.0,
        }
    }

    pub fn border_width(self) -> f32 {
        match self {
            Rounding::Square => 1.5,
            Rounding::Sharp => 1.5,
            Rounding::Rounded => 1.0,
            Rounding::Smooth => 1.0,
            Rounding::Pill => 0.5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Rounding::Square => "Square",
            Rounding::Sharp => "Sharp",
            Rounding::Rounded => "Rounded",
            Rounding::Smooth => "Smooth",
            Rounding::Pill => "Pill",
        }
    }
}

// ── ShadowDepth ───────────────────────────────────────────────────────────────

/// Drop-shadow intensity. `Copy` — safe to move into `'static` closures.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ShadowDepth {
    /// No shadow at all.
    None,
    /// Faint lift — 6 px blur.
    #[default]
    Subtle,
    /// Visible card elevation — 16 px blur.
    Medium,
    /// Strong floating shadow — 28 px blur.
    Elevated,
}

impl ShadowDepth {
    pub fn blur(self) -> f32 {
        match self {
            ShadowDepth::None => 0.0,
            ShadowDepth::Subtle => 6.0,
            ShadowDepth::Medium => 16.0,
            ShadowDepth::Elevated => 28.0,
        }
    }

    pub fn offset(self) -> f32 {
        match self {
            ShadowDepth::None => 0.0,
            ShadowDepth::Subtle => 1.0,
            ShadowDepth::Medium => 4.0,
            ShadowDepth::Elevated => 8.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ShadowDepth::None => "None",
            ShadowDepth::Subtle => "Subtle",
            ShadowDepth::Medium => "Medium",
            ShadowDepth::Elevated => "Elevated",
        }
    }
}

// ── ThemeConfig / AppTheme ────────────────────────────────────────────────────

/// Theme configuration persisted in `GuiSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub family: PaletteFamily,
    pub dark: bool,
    /// `None` → use `Rounding::default()` (Rounded); `Some` → explicit user override.
    #[serde(default)]
    pub rounding: Option<Rounding>,
    /// `None` → use `ShadowDepth::default()` (Subtle); `Some` → explicit user override.
    #[serde(default)]
    pub shadow: Option<ShadowDepth>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            family: PaletteFamily::Crimson,
            dark: true,
            rounding: None,
            shadow: None,
        }
    }
}

impl ThemeConfig {
    /// Resolve into a concrete `AppTheme` ready for rendering.
    pub fn resolve(self) -> AppTheme {
        let palette = self.family.palette(self.dark);
        let rounding = self.rounding.unwrap_or_default();
        let shadow = self.shadow.unwrap_or_default();
        AppTheme {
            palette,
            rounding,
            shadow,
        }
    }
}

/// Fully-resolved theme, computed at render time from `ThemeConfig`.
#[derive(Debug, Clone, Copy)]
pub struct AppTheme {
    pub palette: Palette,
    pub rounding: Rounding,
    pub shadow: ShadowDepth,
}
