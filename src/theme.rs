//! Colour schemes for the syntax highlighter and the UI chrome.

use crate::syntax::{Tok, TOK_COUNT};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<u8>,
    pub bg: Option<u8>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub const fn plain() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub const fn fg(color: u8) -> Self {
        Self {
            fg: Some(color),
            ..Self::plain()
        }
    }

    pub const fn bg(color: u8) -> Self {
        Self {
            bg: Some(color),
            ..Self::plain()
        }
    }

    pub const fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    pub const fn italic(self) -> Self {
        Self {
            italic: true,
            ..self
        }
    }

    pub const fn underline(self) -> Self {
        Self {
            underline: true,
            ..self
        }
    }

    pub const fn with_bg(self, color: u8) -> Self {
        Self {
            bg: Some(color),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
    SolarizedDark,
    SolarizedLight,
    GruvboxDark,
    GruvboxLight,
    Nord,
    Dracula,
    Monokai,
    OneDark,
    TokyoNight,
}

impl ThemeName {
    /// Every built-in theme, in the order they are listed in the docs.
    pub const ALL: [ThemeName; 11] = [
        ThemeName::Dark,
        ThemeName::Light,
        ThemeName::SolarizedDark,
        ThemeName::SolarizedLight,
        ThemeName::GruvboxDark,
        ThemeName::GruvboxLight,
        ThemeName::Nord,
        ThemeName::Dracula,
        ThemeName::Monokai,
        ThemeName::OneDark,
        ThemeName::TokyoNight,
    ];

    /// Canonical value accepted by the `theme` configuration option.
    pub fn name(self) -> &'static str {
        match self {
            ThemeName::Dark => "dark",
            ThemeName::Light => "light",
            ThemeName::SolarizedDark => "solarized-dark",
            ThemeName::SolarizedLight => "solarized-light",
            ThemeName::GruvboxDark => "gruvbox-dark",
            ThemeName::GruvboxLight => "gruvbox-light",
            ThemeName::Nord => "nord",
            ThemeName::Dracula => "dracula",
            ThemeName::Monokai => "monokai",
            ThemeName::OneDark => "one-dark",
            ThemeName::TokyoNight => "tokyo-night",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let key = s.trim().to_ascii_lowercase().replace(['-', '_', ' '], "");
        match key.as_str() {
            "dark" | "default" => Some(ThemeName::Dark),
            "light" => Some(ThemeName::Light),
            "solarizeddark" => Some(ThemeName::SolarizedDark),
            "solarizedlight" => Some(ThemeName::SolarizedLight),
            "gruvbox" | "gruvboxdark" => Some(ThemeName::GruvboxDark),
            "gruvboxlight" => Some(ThemeName::GruvboxLight),
            "nord" => Some(ThemeName::Nord),
            "dracula" => Some(ThemeName::Dracula),
            "monokai" => Some(ThemeName::Monokai),
            "onedark" => Some(ThemeName::OneDark),
            "tokyonight" => Some(ThemeName::TokyoNight),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Theme {
    pub syntax: [Style; TOK_COUNT],
    pub text: Style,
    pub gutter: Style,
    pub gutter_current: Style,
    pub modeline: Style,
    pub modeline_accent: Style,
    pub minibuffer: Style,
    pub prompt: Style,
    pub region: Style,
    pub search: Style,
    pub trailing_ws: Style,
    pub help_heading: Style,
    pub help_key: Style,
    pub help_dim: Style,
    pub fill: Style,
    pub completion: Style,
    pub completion_selected: Style,
    pub spell: Style,
    pub diagnostic_error: Style,
    pub diagnostic_warning: Style,
    pub diagnostic_information: Style,
    pub diagnostic_hint: Style,
    pub tab_bar: Style,
    pub tab_active: Style,
    pub rainbow: [Style; 6],
}

impl Theme {
    pub fn by_name(name: ThemeName) -> Theme {
        match name {
            ThemeName::Dark => Theme::dark(),
            ThemeName::Light => Theme::light(),
            ThemeName::SolarizedDark => Theme::solarized_dark(),
            ThemeName::SolarizedLight => Theme::solarized_light(),
            ThemeName::GruvboxDark => Theme::gruvbox_dark(),
            ThemeName::GruvboxLight => Theme::gruvbox_light(),
            ThemeName::Nord => Theme::nord(),
            ThemeName::Dracula => Theme::dracula(),
            ThemeName::Monokai => Theme::monokai(),
            ThemeName::OneDark => Theme::one_dark(),
            ThemeName::TokyoNight => Theme::tokyo_night(),
        }
    }

    pub fn tok(&self, tok: Tok) -> Style {
        self.syntax[tok as usize]
    }

    pub fn dark() -> Theme {
        let syntax = [
            Style::fg(81).bold(),       // Keyword
            Style::fg(114),             // Type
            Style::fg(179),             // Constant
            Style::fg(222),             // Function
            Style::fg(150),             // String
            Style::fg(244),             // Comment
            Style::fg(180),             // Number
            Style::fg(210),             // Operator
            Style::fg(176),             // Preproc
            Style::fg(117).bold(),      // Heading
            Style::fg(223).italic(),    // Emphasis
            Style::fg(75).underline(),  // Link
            Style::fg(120).with_bg(22), // DiffAdd
            Style::fg(210).with_bg(52), // DiffDel
            Style::fg(81).with_bg(236), // DiffHunk
            Style::fg(246),             // Meta
        ];
        Theme {
            syntax,
            text: Style::plain(),
            gutter: Style::fg(240),
            gutter_current: Style::fg(252).bold(),
            modeline: Style::fg(252).with_bg(236),
            modeline_accent: Style::fg(81).with_bg(236).bold(),
            minibuffer: Style::fg(252).with_bg(234),
            prompt: Style::fg(81).with_bg(234).bold(),
            region: Style::plain().with_bg(237),
            search: Style::fg(231).with_bg(58),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(81).bold(),
            help_key: Style::fg(222),
            help_dim: Style::fg(244),
            fill: Style::fg(238),
            completion: Style::fg(250).with_bg(236),
            completion_selected: Style::fg(231).with_bg(60).bold(),
            spell: Style::fg(203).underline(),
            diagnostic_error: Style::fg(203).underline(),
            diagnostic_warning: Style::fg(221).underline(),
            diagnostic_information: Style::fg(117).underline(),
            diagnostic_hint: Style::fg(244).underline(),
            tab_bar: Style::fg(250).with_bg(236),
            tab_active: Style::fg(231).with_bg(60).bold(),
            rainbow: [
                Style::fg(203).bold(),
                Style::fg(221).bold(),
                Style::fg(117).bold(),
                Style::fg(114).bold(),
                Style::fg(179).bold(),
                Style::fg(141).bold(),
            ],
        }
    }

    pub fn light() -> Theme {
        let syntax = [
            Style::fg(25).bold(),        // Keyword
            Style::fg(28),               // Type
            Style::fg(130),              // Constant
            Style::fg(160),              // Function
            Style::fg(28),               // String
            Style::fg(244),              // Comment
            Style::fg(94),               // Number
            Style::fg(90),               // Operator
            Style::fg(55),               // Preproc
            Style::fg(25).bold(),        // Heading
            Style::fg(90).italic(),      // Emphasis
            Style::fg(26).underline(),   // Link
            Style::fg(22).with_bg(194),  // DiffAdd
            Style::fg(124).with_bg(224), // DiffDel
            Style::fg(25).with_bg(254),  // DiffHunk
            Style::fg(240),              // Meta
        ];
        Theme {
            syntax,
            text: Style::plain(),
            gutter: Style::fg(250),
            gutter_current: Style::fg(236).bold(),
            modeline: Style::fg(235).with_bg(254),
            modeline_accent: Style::fg(25).with_bg(254).bold(),
            minibuffer: Style::fg(235).with_bg(255),
            prompt: Style::fg(25).with_bg(255).bold(),
            region: Style::plain().with_bg(254),
            search: Style::fg(16).with_bg(226),
            trailing_ws: Style::plain().with_bg(217),
            help_heading: Style::fg(25).bold(),
            help_key: Style::fg(160),
            help_dim: Style::fg(244),
            fill: Style::fg(250),
            completion: Style::fg(235).with_bg(253),
            completion_selected: Style::fg(16).with_bg(153).bold(),
            spell: Style::fg(160).underline(),
            diagnostic_error: Style::fg(160).underline(),
            diagnostic_warning: Style::fg(130).underline(),
            diagnostic_information: Style::fg(25).underline(),
            diagnostic_hint: Style::fg(240).underline(),
            tab_bar: Style::fg(235).with_bg(253),
            tab_active: Style::fg(16).with_bg(153).bold(),
            rainbow: [
                Style::fg(160).bold(),
                Style::fg(130).bold(),
                Style::fg(25).bold(),
                Style::fg(28).bold(),
                Style::fg(90).bold(),
                Style::fg(129).bold(),
            ],
        }
    }

    pub fn solarized_dark() -> Theme {
        let syntax = [
            Style::fg(100).bold(),      // Keyword
            Style::fg(136),             // Type
            Style::fg(37),              // Constant
            Style::fg(32),              // Function
            Style::fg(37),              // String
            Style::fg(242),             // Comment
            Style::fg(37),              // Number
            Style::fg(100),             // Operator
            Style::fg(166),             // Preproc
            Style::fg(32).bold(),       // Heading
            Style::fg(136).italic(),    // Emphasis
            Style::fg(32).underline(),  // Link
            Style::fg(100).with_bg(22), // DiffAdd
            Style::fg(160).with_bg(52), // DiffDel
            Style::fg(32).with_bg(235), // DiffHunk
            Style::fg(242),             // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(245),
            gutter: Style::fg(240),
            gutter_current: Style::fg(245).bold(),
            modeline: Style::fg(245).with_bg(235),
            modeline_accent: Style::fg(32).with_bg(235).bold(),
            minibuffer: Style::fg(245).with_bg(234),
            prompt: Style::fg(32).with_bg(234).bold(),
            region: Style::plain().with_bg(236),
            search: Style::fg(234).with_bg(136),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(32).bold(),
            help_key: Style::fg(136),
            help_dim: Style::fg(242),
            fill: Style::fg(238),
            completion: Style::fg(245).with_bg(236),
            completion_selected: Style::fg(234).with_bg(32).bold(),
            spell: Style::fg(160).underline(),
            diagnostic_error: Style::fg(160).underline(),
            diagnostic_warning: Style::fg(136).underline(),
            diagnostic_information: Style::fg(32).underline(),
            diagnostic_hint: Style::fg(242).underline(),
            tab_bar: Style::fg(245).with_bg(236),
            tab_active: Style::fg(234).with_bg(32).bold(),
            rainbow: [
                Style::fg(160).bold(),
                Style::fg(136).bold(),
                Style::fg(32).bold(),
                Style::fg(100).bold(),
                Style::fg(37).bold(),
                Style::fg(62).bold(),
            ],
        }
    }

    pub fn solarized_light() -> Theme {
        let syntax = [
            Style::fg(100).bold(),       // Keyword
            Style::fg(136),              // Type
            Style::fg(37),               // Constant
            Style::fg(32),               // Function
            Style::fg(37),               // String
            Style::fg(247),              // Comment
            Style::fg(37),               // Number
            Style::fg(100),              // Operator
            Style::fg(166),              // Preproc
            Style::fg(32).bold(),        // Heading
            Style::fg(136).italic(),     // Emphasis
            Style::fg(32).underline(),   // Link
            Style::fg(22).with_bg(194),  // DiffAdd
            Style::fg(124).with_bg(224), // DiffDel
            Style::fg(32).with_bg(255),  // DiffHunk
            Style::fg(243),              // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(243),
            gutter: Style::fg(246),
            gutter_current: Style::fg(240).bold(),
            modeline: Style::fg(240).with_bg(254),
            modeline_accent: Style::fg(32).with_bg(254).bold(),
            minibuffer: Style::fg(240).with_bg(255),
            prompt: Style::fg(32).with_bg(255).bold(),
            region: Style::plain().with_bg(254),
            search: Style::fg(230).with_bg(136),
            trailing_ws: Style::plain().with_bg(217),
            help_heading: Style::fg(32).bold(),
            help_key: Style::fg(166),
            help_dim: Style::fg(247),
            fill: Style::fg(246),
            completion: Style::fg(240).with_bg(253),
            completion_selected: Style::fg(16).with_bg(153).bold(),
            spell: Style::fg(160).underline(),
            diagnostic_error: Style::fg(160).underline(),
            diagnostic_warning: Style::fg(136).underline(),
            diagnostic_information: Style::fg(32).underline(),
            diagnostic_hint: Style::fg(240).underline(),
            tab_bar: Style::fg(240).with_bg(253),
            tab_active: Style::fg(16).with_bg(153).bold(),
            rainbow: [
                Style::fg(160).bold(),
                Style::fg(136).bold(),
                Style::fg(32).bold(),
                Style::fg(100).bold(),
                Style::fg(37).bold(),
                Style::fg(62).bold(),
            ],
        }
    }

    pub fn gruvbox_dark() -> Theme {
        let syntax = [
            Style::fg(203).bold(),       // Keyword
            Style::fg(214),              // Type
            Style::fg(175),              // Constant
            Style::fg(142),              // Function
            Style::fg(142),              // String
            Style::fg(245),              // Comment
            Style::fg(175),              // Number
            Style::fg(109),              // Operator
            Style::fg(208),              // Preproc
            Style::fg(214).bold(),       // Heading
            Style::fg(214).italic(),     // Emphasis
            Style::fg(109).underline(),  // Link
            Style::fg(142).with_bg(22),  // DiffAdd
            Style::fg(203).with_bg(52),  // DiffDel
            Style::fg(109).with_bg(237), // DiffHunk
            Style::fg(245),              // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(223),
            gutter: Style::fg(245),
            gutter_current: Style::fg(223).bold(),
            modeline: Style::fg(223).with_bg(237),
            modeline_accent: Style::fg(214).with_bg(237).bold(),
            minibuffer: Style::fg(223).with_bg(235),
            prompt: Style::fg(214).with_bg(235).bold(),
            region: Style::plain().with_bg(239),
            search: Style::fg(235).with_bg(214),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(214).bold(),
            help_key: Style::fg(142),
            help_dim: Style::fg(245),
            fill: Style::fg(243),
            completion: Style::fg(223).with_bg(237),
            completion_selected: Style::fg(235).with_bg(142).bold(),
            spell: Style::fg(203).underline(),
            diagnostic_error: Style::fg(203).underline(),
            diagnostic_warning: Style::fg(214).underline(),
            diagnostic_information: Style::fg(109).underline(),
            diagnostic_hint: Style::fg(245).underline(),
            tab_bar: Style::fg(223).with_bg(237),
            tab_active: Style::fg(235).with_bg(142).bold(),
            rainbow: [
                Style::fg(203).bold(),
                Style::fg(214).bold(),
                Style::fg(142).bold(),
                Style::fg(109).bold(),
                Style::fg(175).bold(),
                Style::fg(208).bold(),
            ],
        }
    }

    pub fn gruvbox_light() -> Theme {
        let syntax = [
            Style::fg(124).bold(),       // Keyword
            Style::fg(136),              // Type
            Style::fg(126),              // Constant
            Style::fg(100),              // Function
            Style::fg(100),              // String
            Style::fg(243),              // Comment
            Style::fg(126),              // Number
            Style::fg(24),               // Operator
            Style::fg(130),              // Preproc
            Style::fg(136).bold(),       // Heading
            Style::fg(136).italic(),     // Emphasis
            Style::fg(24).underline(),   // Link
            Style::fg(22).with_bg(194),  // DiffAdd
            Style::fg(124).with_bg(224), // DiffDel
            Style::fg(24).with_bg(230),  // DiffHunk
            Style::fg(243),              // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(237),
            gutter: Style::fg(243),
            gutter_current: Style::fg(237).bold(),
            modeline: Style::fg(237).with_bg(187),
            modeline_accent: Style::fg(136).with_bg(187).bold(),
            minibuffer: Style::fg(237).with_bg(230),
            prompt: Style::fg(136).with_bg(230).bold(),
            region: Style::plain().with_bg(187),
            search: Style::fg(230).with_bg(136),
            trailing_ws: Style::plain().with_bg(217),
            help_heading: Style::fg(136).bold(),
            help_key: Style::fg(130),
            help_dim: Style::fg(243),
            fill: Style::fg(243),
            completion: Style::fg(237).with_bg(223),
            completion_selected: Style::fg(230).with_bg(100).bold(),
            spell: Style::fg(124).underline(),
            diagnostic_error: Style::fg(124).underline(),
            diagnostic_warning: Style::fg(130).underline(),
            diagnostic_information: Style::fg(24).underline(),
            diagnostic_hint: Style::fg(243).underline(),
            tab_bar: Style::fg(237).with_bg(223),
            tab_active: Style::fg(230).with_bg(100).bold(),
            rainbow: [
                Style::fg(124).bold(),
                Style::fg(136).bold(),
                Style::fg(100).bold(),
                Style::fg(24).bold(),
                Style::fg(126).bold(),
                Style::fg(130).bold(),
            ],
        }
    }

    pub fn nord() -> Theme {
        let syntax = [
            Style::fg(110).bold(),       // Keyword
            Style::fg(109),              // Type
            Style::fg(139),              // Constant
            Style::fg(110),              // Function
            Style::fg(108),              // String
            Style::fg(244),              // Comment
            Style::fg(139),              // Number
            Style::fg(110),              // Operator
            Style::fg(173),              // Preproc
            Style::fg(110).bold(),       // Heading
            Style::fg(139).italic(),     // Emphasis
            Style::fg(110).underline(),  // Link
            Style::fg(108).with_bg(22),  // DiffAdd
            Style::fg(167).with_bg(52),  // DiffDel
            Style::fg(110).with_bg(238), // DiffHunk
            Style::fg(244),              // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(253),
            gutter: Style::fg(243),
            gutter_current: Style::fg(253).bold(),
            modeline: Style::fg(253).with_bg(239),
            modeline_accent: Style::fg(110).with_bg(239).bold(),
            minibuffer: Style::fg(253).with_bg(237),
            prompt: Style::fg(110).with_bg(237).bold(),
            region: Style::plain().with_bg(240),
            search: Style::fg(237).with_bg(180),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(110).bold(),
            help_key: Style::fg(173),
            help_dim: Style::fg(244),
            fill: Style::fg(242),
            completion: Style::fg(253).with_bg(239),
            completion_selected: Style::fg(237).with_bg(110).bold(),
            spell: Style::fg(167).underline(),
            diagnostic_error: Style::fg(167).underline(),
            diagnostic_warning: Style::fg(180).underline(),
            diagnostic_information: Style::fg(110).underline(),
            diagnostic_hint: Style::fg(244).underline(),
            tab_bar: Style::fg(253).with_bg(239),
            tab_active: Style::fg(237).with_bg(110).bold(),
            rainbow: [
                Style::fg(167).bold(),
                Style::fg(180).bold(),
                Style::fg(108).bold(),
                Style::fg(110).bold(),
                Style::fg(139).bold(),
                Style::fg(173).bold(),
            ],
        }
    }

    pub fn dracula() -> Theme {
        let syntax = [
            Style::fg(212).bold(),       // Keyword
            Style::fg(117),              // Type
            Style::fg(141),              // Constant
            Style::fg(84),               // Function
            Style::fg(228),              // String
            Style::fg(61),               // Comment
            Style::fg(141),              // Number
            Style::fg(212),              // Operator
            Style::fg(215),              // Preproc
            Style::fg(212).bold(),       // Heading
            Style::fg(228).italic(),     // Emphasis
            Style::fg(117).underline(),  // Link
            Style::fg(84).with_bg(22),   // DiffAdd
            Style::fg(203).with_bg(52),  // DiffDel
            Style::fg(117).with_bg(239), // DiffHunk
            Style::fg(61),               // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(255),
            gutter: Style::fg(61),
            gutter_current: Style::fg(255).bold(),
            modeline: Style::fg(255).with_bg(239),
            modeline_accent: Style::fg(212).with_bg(239).bold(),
            minibuffer: Style::fg(255).with_bg(236),
            prompt: Style::fg(212).with_bg(236).bold(),
            region: Style::plain().with_bg(239),
            search: Style::fg(236).with_bg(228),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(212).bold(),
            help_key: Style::fg(84),
            help_dim: Style::fg(61),
            fill: Style::fg(239),
            completion: Style::fg(255).with_bg(239),
            completion_selected: Style::fg(236).with_bg(141).bold(),
            spell: Style::fg(203).underline(),
            diagnostic_error: Style::fg(203).underline(),
            diagnostic_warning: Style::fg(228).underline(),
            diagnostic_information: Style::fg(117).underline(),
            diagnostic_hint: Style::fg(61).underline(),
            tab_bar: Style::fg(255).with_bg(239),
            tab_active: Style::fg(236).with_bg(141).bold(),
            rainbow: [
                Style::fg(212).bold(),
                Style::fg(228).bold(),
                Style::fg(84).bold(),
                Style::fg(117).bold(),
                Style::fg(141).bold(),
                Style::fg(215).bold(),
            ],
        }
    }

    pub fn monokai() -> Theme {
        let syntax = [
            Style::fg(197).bold(),      // Keyword
            Style::fg(81),              // Type
            Style::fg(141),             // Constant
            Style::fg(148),             // Function
            Style::fg(186),             // String
            Style::fg(242),             // Comment
            Style::fg(141),             // Number
            Style::fg(197),             // Operator
            Style::fg(208),             // Preproc
            Style::fg(197).bold(),      // Heading
            Style::fg(186).italic(),    // Emphasis
            Style::fg(81).underline(),  // Link
            Style::fg(148).with_bg(22), // DiffAdd
            Style::fg(197).with_bg(52), // DiffDel
            Style::fg(81).with_bg(236), // DiffHunk
            Style::fg(242),             // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(255),
            gutter: Style::fg(242),
            gutter_current: Style::fg(255).bold(),
            modeline: Style::fg(255).with_bg(236),
            modeline_accent: Style::fg(197).with_bg(236).bold(),
            minibuffer: Style::fg(255).with_bg(235),
            prompt: Style::fg(197).with_bg(235).bold(),
            region: Style::plain().with_bg(237),
            search: Style::fg(235).with_bg(186),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(197).bold(),
            help_key: Style::fg(148),
            help_dim: Style::fg(242),
            fill: Style::fg(240),
            completion: Style::fg(255).with_bg(236),
            completion_selected: Style::fg(235).with_bg(148).bold(),
            spell: Style::fg(197).underline(),
            diagnostic_error: Style::fg(197).underline(),
            diagnostic_warning: Style::fg(208).underline(),
            diagnostic_information: Style::fg(81).underline(),
            diagnostic_hint: Style::fg(242).underline(),
            tab_bar: Style::fg(255).with_bg(236),
            tab_active: Style::fg(235).with_bg(148).bold(),
            rainbow: [
                Style::fg(197).bold(),
                Style::fg(208).bold(),
                Style::fg(148).bold(),
                Style::fg(81).bold(),
                Style::fg(141).bold(),
                Style::fg(186).bold(),
            ],
        }
    }

    pub fn one_dark() -> Theme {
        let syntax = [
            Style::fg(176).bold(),      // Keyword
            Style::fg(73),              // Type
            Style::fg(180),             // Constant
            Style::fg(75),              // Function
            Style::fg(108),             // String
            Style::fg(241),             // Comment
            Style::fg(180),             // Number
            Style::fg(73),              // Operator
            Style::fg(176),             // Preproc
            Style::fg(75).bold(),       // Heading
            Style::fg(180).italic(),    // Emphasis
            Style::fg(75).underline(),  // Link
            Style::fg(108).with_bg(22), // DiffAdd
            Style::fg(168).with_bg(52), // DiffDel
            Style::fg(75).with_bg(236), // DiffHunk
            Style::fg(241),             // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(249),
            gutter: Style::fg(241),
            gutter_current: Style::fg(249).bold(),
            modeline: Style::fg(249).with_bg(236),
            modeline_accent: Style::fg(75).with_bg(236).bold(),
            minibuffer: Style::fg(249).with_bg(235),
            prompt: Style::fg(75).with_bg(235).bold(),
            region: Style::plain().with_bg(238),
            search: Style::fg(235).with_bg(180),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(75).bold(),
            help_key: Style::fg(108),
            help_dim: Style::fg(241),
            fill: Style::fg(238),
            completion: Style::fg(249).with_bg(236),
            completion_selected: Style::fg(235).with_bg(75).bold(),
            spell: Style::fg(168).underline(),
            diagnostic_error: Style::fg(168).underline(),
            diagnostic_warning: Style::fg(180).underline(),
            diagnostic_information: Style::fg(75).underline(),
            diagnostic_hint: Style::fg(241).underline(),
            tab_bar: Style::fg(249).with_bg(236),
            tab_active: Style::fg(235).with_bg(75).bold(),
            rainbow: [
                Style::fg(168).bold(),
                Style::fg(180).bold(),
                Style::fg(108).bold(),
                Style::fg(75).bold(),
                Style::fg(176).bold(),
                Style::fg(73).bold(),
            ],
        }
    }

    pub fn tokyo_night() -> Theme {
        let syntax = [
            Style::fg(141).bold(),       // Keyword
            Style::fg(117),              // Type
            Style::fg(215),              // Constant
            Style::fg(111),              // Function
            Style::fg(149),              // String
            Style::fg(60),               // Comment
            Style::fg(215),              // Number
            Style::fg(111),              // Operator
            Style::fg(204),              // Preproc
            Style::fg(111).bold(),       // Heading
            Style::fg(179).italic(),     // Emphasis
            Style::fg(111).underline(),  // Link
            Style::fg(149).with_bg(22),  // DiffAdd
            Style::fg(204).with_bg(52),  // DiffDel
            Style::fg(111).with_bg(236), // DiffHunk
            Style::fg(60),               // Meta
        ];
        Theme {
            syntax,
            text: Style::fg(189),
            gutter: Style::fg(60),
            gutter_current: Style::fg(189).bold(),
            modeline: Style::fg(189).with_bg(236),
            modeline_accent: Style::fg(111).with_bg(236).bold(),
            minibuffer: Style::fg(189).with_bg(234),
            prompt: Style::fg(111).with_bg(234).bold(),
            region: Style::plain().with_bg(238),
            search: Style::fg(234).with_bg(179),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(111).bold(),
            help_key: Style::fg(149),
            help_dim: Style::fg(60),
            fill: Style::fg(238),
            completion: Style::fg(189).with_bg(236),
            completion_selected: Style::fg(234).with_bg(111).bold(),
            spell: Style::fg(204).underline(),
            diagnostic_error: Style::fg(204).underline(),
            diagnostic_warning: Style::fg(179).underline(),
            diagnostic_information: Style::fg(111).underline(),
            diagnostic_hint: Style::fg(60).underline(),
            tab_bar: Style::fg(189).with_bg(236),
            tab_active: Style::fg(234).with_bg(111).bold(),
            rainbow: [
                Style::fg(204).bold(),
                Style::fg(179).bold(),
                Style::fg(149).bold(),
                Style::fg(111).bold(),
                Style::fg(141).bold(),
                Style::fg(215).bold(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_have_a_style_for_every_token() {
        let all = [
            Tok::Keyword,
            Tok::Type,
            Tok::Constant,
            Tok::Function,
            Tok::String,
            Tok::Comment,
            Tok::Number,
            Tok::Operator,
            Tok::Preproc,
            Tok::Heading,
            Tok::Emphasis,
            Tok::Link,
            Tok::DiffAdd,
            Tok::DiffDel,
            Tok::DiffHunk,
            Tok::Meta,
        ];
        assert_eq!(all.len(), TOK_COUNT);
        for name in ThemeName::ALL {
            let theme = Theme::by_name(name);
            for tok in all {
                assert!(
                    theme.tok(tok).fg.is_some() || theme.tok(tok).bg.is_some(),
                    "{} has no style for {tok:?}",
                    name.name()
                );
            }
        }
    }

    #[test]
    fn theme_name_parsing() {
        assert_eq!(ThemeName::parse("Dark"), Some(ThemeName::Dark));
        assert_eq!(ThemeName::parse("light"), Some(ThemeName::Light));
        assert_eq!(
            ThemeName::parse("Solarized Dark"),
            Some(ThemeName::SolarizedDark)
        );
        assert_eq!(
            ThemeName::parse("solarized_light"),
            Some(ThemeName::SolarizedLight)
        );
        assert_eq!(ThemeName::parse("gruvbox"), Some(ThemeName::GruvboxDark));
        assert_eq!(
            ThemeName::parse("Gruvbox Light"),
            Some(ThemeName::GruvboxLight)
        );
        assert_eq!(ThemeName::parse("nord"), Some(ThemeName::Nord));
        assert_eq!(ThemeName::parse("dracula"), Some(ThemeName::Dracula));
        assert_eq!(ThemeName::parse("monokai"), Some(ThemeName::Monokai));
        assert_eq!(ThemeName::parse("one_dark"), Some(ThemeName::OneDark));
        assert_eq!(ThemeName::parse("tokyo-night"), Some(ThemeName::TokyoNight));
        assert_eq!(ThemeName::parse("nope"), None);
    }

    #[test]
    fn every_theme_name_round_trips() {
        for name in ThemeName::ALL {
            assert_eq!(ThemeName::parse(name.name()), Some(name));
        }
    }
}
