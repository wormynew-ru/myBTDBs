# CHANGELOG.md — История изменений

## [0.2.0] — Реализация каркаса и игровой логики на Rust + Bevy 0.15
### Добавлено
- Точки входа `src/lib.rs` (`#[bevy_main]`) и `src/main.rs` с камерой `1280×720` (`ScalingMode::AutoMin`).
- Слой `src/core/`: состояния `GameState`, маркеры `Side::Player`/`Side::Bot`, события доменов, унифицированный ввод `PointerInput` (мышь и тачскрин).
- Слой `src/data/`: конфиги слоёв шаров и наборов отправки (`bloons_cfg.rs`), 4 башен с ветками апгрейдов 2×3 (`towers_cfg.rs`) и 3 симметричных карт (`maps_cfg.rs`).
- Слой `src/plugins/`: изолированные плагины `map`, `bloons`, `towers`, `economy` (выплаты Eco каждые 6 сек), `ai` (Utility AI с оценкой угроз, контр-башнями и рашами).
- Слой `src/ui/`: экраны `menu`, `hud`, `build_bar`, `upgrade_panel`, `game_over` на `bevy_ui` (кнопки ≥ 48×48).
- Утилиты `src/utils/mod.rs` для геометрии маршрутов и проверки зон строительства.

## — Инициализация архитектуры
### Добавлено
- Утверждена спецификация `TZ.md` и архитектурная конституция `CLAUDE.md` под Rust + Bevy 0.15 (PC + Android).
- Создана карта проекта `ROADMAP.md` с разбиением на слои `core`, `data`, `plugins`, `ui`, `utils`.
- Настроен CI/CD в `.github/workflows/build-android.yml` для облачной сборки `.apk` под `aarch64-linux-android`.
