# CHANGELOG.md — История изменений

## [0.3.0] — Спрайты 16×16, очередь отправки, Шаромолка и Деревня
### Добавлено
- Процедурные пиксель-арт текстуры `16×16` (`src/plugins/sprites.rs`) с `ImageSampler::nearest()` для шаров, башен, снарядов, порталов и частиц; создана спецификация `req.md`.
- Очередь отправки шаров `SendPackQueue` (FIFO по целевой стороне без наложения пачек) и отображение очереди в HUD (`Q:N`).
- Новые слои и веса шаров: `Rainbow`, `MOAB`, `BFB`, `ZOMG` + набор отправки `1x MOAB`.
- Башня **Bloonchipper (Шаромолка)** (`src/plugins/towers/chipper.rs`): 4-фазный FSM (`Scanning -> Pulling -> Shredding -> Ejecting`), `weight`/`max_capacity`, апгрейды `2×4` (включая `0-3 Triple Barrel`, `4-0 MOAB Shredder`, активную способность `0-4 Supa-Vac`), вибрация и частицы выхлопа.
- Башня **Monkey Village (Деревня)**: аура скорости, дальности, пробития, детекции камуфляжа (`0-2 Radar Scanner`) и свинца (`0-3 MIB`).

## [0.2.1] — Возврат кодовой базы Rust и правило «Только Rust»
### Изменено
- Восстановлена полная кодовая база игры на **Rust + Bevy 0.15** (`Cargo.toml`, `src/`).
- Восстановлена web-заглушка (`index.html` + `package.json`) для порта 3000.
- В `CLAUDE.md` добавлено жёсткое правило: проект разрабатывается исключительно на языке Rust.

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
- Утверждена спецификация `TZ.md` и архитектурная конституция `CLAUDE.md` под Rust + Bevy 0.15 (P C + Android).
- Создана карта проекта `ROADMAP.md` с разбиением на слои `core`, `data`, `plugins`, `ui`, `utils`.
- Настроен CI/CD в `.github/workflows/build-android.yml` для облачной сборки `.apk` под `aarch64-linux-android`.
