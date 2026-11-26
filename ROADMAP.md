# ROADMAP.md — Карта модулей и файлов проекта

## Корневые файлы
- `Cargo.toml` — зависимости Bevy 0.15 и метаданные сборки Android (`cargo-apk`).
- `.github/workflows/build-android.yml` — CI/CD пайплайн автоматической сборки `.apk` на GitHub.
- `CLAUDE.md` — архитектурные правила и ограничения.
- `TZ.md` — функциональное техническое задание.
- `CHANGELOG.md` — история изменений проекта.
- `src/main.rs` — точка входа для ПК (вызывает `lib::main()`).
- `src/lib.rs` — главная точка входа (`#[bevy_main]`), настройка камеры, окна и регистрация плагинов.

## Ядро (`src/core/`)
- `src/core/mod.rs` — реэкспорт модулей ядра.
- `src/core/state.rs` — состояния игры (`GameState`: `MainMenu`, `Playing`, `GameOver`) и настройки матча.
- `src/core/types.rs` — базовые типы и компоненты (`Side::Player`, `Side::Bot`, `Health`, `Position`).
- `src/core/events.rs` — все игровые события (`SpawnBloonEvent`, `BuildTowerEvent`, `UpgradeTowerEvent`, `BloonLeakedEvent`).
- `src/core/input.rs` — унифицированный ввод (`PointerInput`) для мыши и сенсора.

## Данные и баланс (`src/data/`)
- `src/data/mod.rs` — реэкспорт конфигов.
- `src/data/bloons_cfg.rs` — характеристики слоёв шаров и наборов для отправки (цена, прирост Eco).
- `src/data/towers_cfg.rs` — характеристики башен, цены и дерево апгрейдов (2×3).
- `src/data/maps_cfg.rs` — координаты маршрутов (waypoints) для обеих половин экрана.

## Игровые домены (`src/plugins/`)
- `src/plugins/mod.rs` — сборка всех игровых плагинов.
- `src/plugins/map.rs` — отрисовка полей, дорог и проверка зон постройки.
- `src/plugins/bloons.rs` — спавн волн, движение по маршруту, снятие слоёв и деление шаров.
- `src/plugins/towers.rs` — поиск целей по приоритету, стрельба, полёт снарядов и коллизии.
- `src/plugins/economy.rs` — таймер выплат `Eco`, списание денег, учёт жизней и проверка конца матча.
- `src/plugins/ai.rs` — Utility AI бота (оценка угроз, строительство, апгрейды, экономика и раши).

## Интерфейс (`src/ui/`)
- `src/ui/mod.rs` — реэкспорт UI-плагина.
- `src/ui/menu.rs` — главное меню и выбор сложности.
- `src/ui/hud.rs` — верхняя панель ресурсов игрока и бота.
- `src/ui/build_bar.rs` — кнопки покупки башен и отправки шаров.
- `src/ui/upgrade_panel.rs` — панель выбранной башни (апгрейды, смена цели, продажа).
- `src/ui/game_over.rs` — экран результатов матча и рестарт.

## Утилиты (`src/utils/`)
- `src/utils/mod.rs` — чистые математические хелперы (расстояние до отрезка, пересечение кругов).
