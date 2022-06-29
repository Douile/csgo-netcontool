macro_rules! valued_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident ($t:ty) {
            $(
            $k:ident = $v:pat
            ),*$(,)+
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $(
                $k,
            )*
        }

        impl $name {
            pub fn try_from(value: $t) -> Option<$name> {
                match value {
                    $(
                        $v => Some($name::$k),
                    )*
                    _ => None,
                }
            }
        }
    }
}

valued_enum!(
    #[derive(Debug, Clone)]
    pub enum GameType (u8) {
    Classic = 0,
    GunGame = 1,
    Training = 2,
    Custom = 3,
    Cooperative = 4,
    Skirmish = 5,
    FreeForAll = 6,
}
);

valued_enum!(
    #[derive(Debug, Clone)]
    pub enum GameMode ((GameType, u8)) {
        Casual = (GameType::Classic, 0),
        Competitive = (GameType::Classic, 1),
        ScrimComp2v2 = (GameType::Classic, 2),
        ScrimComp5v5 = (GameType::Classic, 3),
        GunGameProgressive = (GameType::GunGame, 0),
        GunGameTrBomb = (GameType::GunGame, 1),
        Deathmatch = (GameType::GunGame, 2),
        Training = (GameType::Training, 0),
        Custom = (GameType::Custom, 0),
        Cooperative = (GameType::Cooperative, 0),
        CoopMission = (GameType::Cooperative, 1),
        Skirmish = (GameType::Skirmish, 0),
        Survival = (GameType::FreeForAll, 0),
    }
);

impl ToString for GameMode {
    fn to_string(&self) -> String {
        String::from(match self {
            GameMode::Casual => "Casual",
            GameMode::Competitive => "Competitive",
            GameMode::ScrimComp2v2 => "Wingman",
            GameMode::Custom => "Custom",
            _ => "",
        })
    }
}
