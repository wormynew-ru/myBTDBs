# ROADMAP.md — Карта модулей и файлов проекта

## Корневые файлы
- `Cargo.toml` — зависимости Bevy 0.15 и метаданные сборки Android (`cargo-apk`).
- `.github/workflows/build-android.yml` — CI/CD пайплайн автоматической сборки `.apk` на GitHub.
- `CLAUDE.md` — архитектурные правила и ограничения.
- `TZ.md` — функциональное техническое задание.
- `CHANGELOG.md` — история изменений проекта.
- `req.md` — спецификация спрайтов `16×16` (имена, размеры, цвета, ориентация).
- `src/main.rs` — точка входа для ПК (вызывает `lib::main()`).
- `src/lib.rs` — главная точка входа (`#[bevy_main]`), настройка камеры, окна и регистрация плагинов.

## Ядро (`src/core/`)
- `src/core/mod.rs` — реэкспорт модулей ядра.
- `src/core/state.rs` — состояния игры (`GameState`: `MainMenu`, `Playing`, `GameOver`) и настройки матча.
- `src/core/types.rs` — базовые типы и компоненты (`Side::Player`, `Side::Bot`, `Health`, `Position`, `BloonLayer`, `TowerType`).
- `src/core/events.rs` — все игровые события (`SpawnBloonEvent`, `BuildTowerEvent`, `UpgradeTowerEvent`, `ActivateAbilityEvent`, `BloonLeakedEvent`).
- `src/core/input.rs` — унифицированный ввод (`PointerInput`) для мыши и сенсора.

## Данные и баланс (`src/data/`)
- `src/data/mod.rs` — реэкспорт конфигов.
- `src/data/bloons_cfg.rs` — характеристики слоёв и весов шаров (`Red`..`ZOMG`), наборов отправки (`SEND_PACKS`).
- `src/data/towers_cfg.rs` — характеристики башен, цены и дерево апгрейдов (2×3 / 2×4).
- `src/data/tower_stats.rs` — расчёт эффективных статов и баффов от `Monkey Village`.
- `src/data/maps_cfg.rs` — координаты маршрутов (waypoints) для обеих половин экрана.

## Игровые домены (`src/plugins/`)
- `src/plugins/mod.rs` — сборка всех игровых плагинов.
- `src/plugins/sprites/mod.rs` — генератор процедурных пиксель-арт текстур шаров и накладок `16×16` (`ImageSampler::nearest()`).
- `src/plugins/sprites/towers_px.rs` — генератор `16×16` спрайтов башен (включая все тиры Шаромолки), снарядов и порталов.
- `src/plugins/map.rs` — отрисовка полей, дорог, порталов и проверка зон постройки.
- `src/plugins/bloons.rs` — спавн волн, очередь отправки `SendPackQueue`, движение по маршруту, снятие слоёв.
- `src/plugins/towers/mod.rs` — постройка/апгрейд/продажа башен, аура деревни и активация способностей.
- `src/plugins/towers/combat.rs` — поиск целей по приоритету, стрельба, полёт снарядов и коллизии.
- `src/plugins/towers/chipper.rs` — конечный автомат Шаромолки (`Scanning -> Pulling -> Shredding -> Ejecting`), вибрация и спавн частиц.
- `src/plugins/towers/chipper_fx.rs` — активная способность `Supa-Vac` и обновление частиц выхлопа.
- `src/plugins/towers/spikes.rs` — генерация кучек шипов Фабрики шипов (`SpikePile`), коллизии с шарами, взрывы `4-0 Spiked Mines` и ковёр `0-4 Spike Storm`.
- `src/plugins/towers/spikes_fx.rs` — визуальный эффект огненного взрыва `Spiked Mines` и послойное снятие урона шипами.
- `src/plugins/economy.rs` — таймер выплат `Eco`, постановка отправок в очередь, учёт жизней и конца матча.
- `src/plugins/ai.rs` — Utility AI бота (оценка угроз, строительство, установка Фабрики шипов у выхода, апгрейды, экономика и раши).

## Интерфейс (`src/ui/`)
- `src/ui/mod.rs` — реэкспорт UI-плагина.
- `src/ui/menu.rs` — главное меню и выбор сложности.
- `src/ui/hud.rs` — верхняя панель ресурсов игрока и бота.
- `src/ui/build_bar.rs` — кнопки покупки башен и отправки шаров.
- `src/ui/upgrade_panel.rs` — панель выбранной башни (апгрейды, смена цели, продажа).
- `src/ui/game_over.rs` — экран результатов матча и рестарт.

## Утилиты (`src/utils/`)
- `src/utils/mod.rs` — чистые математические хелперы (расстояние до отрезка, пересечение кругов).
