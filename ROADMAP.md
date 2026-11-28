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
- `src/core/state.rs` — состояния игры (`GameState`: `MainMenu`, `Playing`, `GameOver`), режимы игры (`GameMode::Standard` 5 башен / `GameMode::Turbo` 6 башен) и колоды (`player_loadout`, `bot_loadout`).
- `src/core/types.rs` — базовые типы и компоненты (`Side::Player`, `Side::Bot`, `Health`, `Position`, `BloonLayer`, `TowerType` включая 17 башен, `GlueStatus`).
- `src/core/events.rs` — все игровые события (`SpawnBloonEvent`, `BuildTowerEvent`, `UpgradeTowerEvent`, `ActivateAbilityEvent`, `BloonLeakedEvent`, `BloonPoppedEvent` с `from_send`/`is_moab`/`mirage_flash`).
- `src/core/input.rs` — унифицированный ввод (`PointerInput`) для мыши и сенсора.

## Данные и баланс (`src/data/`)
- `src/data/mod.rs` — реэкспорт конфигов.
- `src/data/bloons_cfg.rs` — характеристики слоёв и весов шаров (`Red`..`ZOMG`), наборов отправки (`SEND_PACKS`) и поздний разгон после 30-й волны (`+22% HP`, `+3%` скорости за волну).
- `src/data/towers_cfg.rs` — список `ALL_TOWERS` (17 башен), базовые характеристики, цены и дерево апгрейдов.
- `src/data/towers_cfg_extra.rs` — конфиги поздних, экономических и тактических башен (`Commander`, `MilitaryBase`, `CrookBoss`, `Farm`, `GlueGunner`, `SoulEater`, `Necromancer`, `Saboteur`, `ScrapCollector`, `MirageGenerator`).
- `src/data/tower_stats.rs` — расчёт эффективных статов, баффов от `Monkey Village`, `Commander` и `ScrapCollector`, скидок от `Farm`.
- `src/data/tower_stats_extra.rs` — расчёт статов поздних и тактических башен (`MilitaryBase`, `CrookBoss`, `Farm`, `GlueGunner`, `SoulEater`, `Necromancer`, `Saboteur`, `ScrapCollector`, `MirageGenerator`).
- `src/data/maps_cfg.rs` — координаты маршрутов (waypoints) для обеих половин экрана.

## Игровые домены (`src/plugins/`)
- `src/plugins/mod.rs` — сборка всех игровых плагинов.
- `src/plugins/sprites/mod.rs` — генератор процедурных пиксель-арт текстур шаров и накладок `16×16` (`ImageSampler::nearest()`).
- `src/plugins/sprites/towers_px.rs` — генератор `16×16` спрайтов базовых башен (включая Шаромолку и Ферму), снарядов и порталов.
- `src/plugins/sprites/units_px.rs` — генератор `16×16` спрайтов Командира, Военной базы, техники, Босса Мафии и головорезов.
- `src/plugins/sprites/new_towers_px.rs` — генератор `16×16` спрайтов Клеевой обезьяны (`glue_gunner.png`, `glue_gunner_liquefier.png`), Пожирателя душ (`soul_eater.png`), Некроманта (`necromancer.png`), призрачных юнитов (`ghost_bloon.png`, `ghost_knight.png`, `rift_portal.png`), клея и тёмных сфер.
- `src/plugins/sprites/tactical_px.rs` — генератор `16×16` спрайтов Саботажника (`saboteur.png`), Сборщика лома (`scrap_collector.png`), Генератора миражей (`mirage_generator.png`) и помех (`jam_static.png`).
- `src/plugins/map.rs` — отрисовка полей, дорог, порталов и проверка зон постройки.
- `src/plugins/bloons.rs` — компонент `Bloon` (включая `from_send` и `is_mirage`), движение по маршруту (включая замедление клеем и скорость призраков/миражей) и синхронизация оверлея `GlueOverlay`.
- `src/plugins/bloons_spawn.rs` — таймер волн (с учётом режима `Standard`/`Turbo`), очереди спавна/отправки и усиление дирижаблей `×1.8 HP` с 35-й волны.
- `src/plugins/towers/mod.rs` — постройка/апгрейд/продажа башен (с блокировкой при `jammed_timer > 0`), активация способностей (`Supa-Vac`, `Glue Striker`, `Jammer`).
- `src/plugins/towers/buffs.rs` — комбинированная аура `Monkey Village`, `Commander` и `ScrapCollector`, а также дебафф ослепления (`×1/3` дальности).
- `src/plugins/towers/combat.rs` — поиск целей по приоритету (с приоритетом несклеенных для `GlueGunner` и притяжением `Strong` на `is_mirage`) и стрельба башен.
- `src/plugins/towers/combat_proj.rs` — полёт снарядов, коллизии, наложение клея, бонусный урон по `Ceramic`/`MOAB` и доход `Golden Boss`.
- `src/plugins/towers/commander.rs` — периодический `Call to Arms` и 2 телохранителя Командира.
- `src/plugins/towers/crook_boss.rs` — призыв головорезов Босса Мафии (`Crook`, `TommyGoon`, `MafiaEnforcer`) с выхода маршрута.
- `src/plugins/towers/farm.rs` — тики сбора урожая Фермы (`FarmState`), дивиденды `0-4` и всплывающий зелёный текст дохода (`FarmIncomeFx`).
- `src/plugins/towers/glue.rs` — статус `GluePayload`, кислотный урон `Corrosive`/`Dissolver`/`Liquefier` и визуальная вспышка `GlueStrikerFx`.
- `src/plugins/towers/military_base.rs` — спавн боевой техники `MilitaryBase` с выхода маршрута и авиаудар `0-4`.
- `src/plugins/towers/mirage.rs` — `MirageGenerator`: спавн `1 HP` голограмм дирижаблей (`MirageMoab`/`Bfb`/`Zomg`) на дорогу противника и ослепляющая вспышка `MirageFlashFx` при разрушении.
- `src/plugins/towers/necromancer.rs` — сбор душ убитых шаров, открытие разломов `RiftPortalFx` на дороге противника и спавн `GhostBloon`/`GhostKnight`.
- `src/plugins/towers/saboteur.rs` — `Saboteur`: импульсный AoE-урон, замедление `0-1..0-4`, способность ослепления и блокировки улучшений дорогих башен противника, анимация помех `JamStaticFx`.
- `src/plugins/towers/scrap_collector.rs` — `ScrapCollector`: кэшбэк за лопнутые вражеские отправки (`from_send`) и нагрев ауры скорости атаки до `+30%`.
- `src/plugins/towers/soul_eater.rs` — поглощение слоёв тотемом `SoulEater` (`350`) и эволюция в случайную max-башню из колоды владельца (`SoulEvolveFx`).
- `src/plugins/towers/units.rs` — логика юнитов `PathUnit` на дороге (замедление до 35% скорости при стрельбе на ходу, поворот к цели, гранаты и таран).
- `src/plugins/towers/units_fx.rs` — послойный урон техники/головорезов и визуальные эффекты взрывов `UnitExplosionFx`.
- `src/plugins/towers/chipper.rs` — конечный автомат Шаромолки (`Scanning -> Pulling -> Shredding -> Ejecting`) и вибрация.
- `src/plugins/towers/chipper_fx.rs` — активная способность `Supa-Vac`, хелперы снятия слоёв/попа и частицы выхлопа `ShredParticle`.
- `src/plugins/towers/spikes.rs` — генерация кучек шипов Фабрики шипов (`SpikePile`), коллизии с шарами, взрывы `4-0 Spiked Mines` и ковёр `0-4 Spike Storm` (28 с).
- `src/plugins/towers/spikes_fx.rs` — визуальный эффект огненного взрыва `Spiked Mines` и послойное снятие урона шипами.
- `src/plugins/economy.rs` — стартовый капитал/интервалы режима `Standard`/`Turbo`, выплаты `Eco`, скидки Ферм, учёт жизней и конца матча.
- `src/plugins/ai.rs` — Utility AI бота (оценка угроз, игра из слепой колоды `bot_loadout`, активация `Supa-Vac`/`Glue Striker`/`Jammer`, контр-спам `ScrapCollector`, перевод анти-MOAB башен на `Strong`).
- `src/plugins/ai_loadout.rs` — генерация слепой колоды бота (включая тактические башни), подбор башен/апгрейдов и пачек раша/Eco.
- `src/plugins/ai_spots.rs` — оценка и подбор оптимальных позиций для боевых, аура- и оффроуд-башен бота (`pick_spot_for_tower`).

## Интерфейс (`src/ui/`)
- `src/ui/mod.rs` — реэкспорт UI-плагина.
- `src/ui/menu.rs` — главное меню, выбор сложности, режима игры (`Standard`/`Turbo`), карты и подтверждение старта.
- `src/ui/menu_loadout.rs` — сетка выбора колоды башен (17 башен по 6 в ряд, 5 или 6 слотов) и кнопка случайной генерации `Random`.
- `src/ui/hud.rs` — верхняя панель ресурсов игрока и бота (с индикатором `[TURBO]`).
- `src/ui/build_bar.rs` — нижняя панель: вся колода башен игрока в один ряд и пагинированная отправка шаров (`[1/2]`).
- `src/ui/upgrade_panel.rs` — панель выбранной башни (кнопки апгрейдов, цели, способностей, продажи).
- `src/ui/upgrade_panel_sync.rs` — синхронизация текста и видимости панели апгрейдов (индикатор `[JAM]`, способность `Jammer`, кэшбэк `ScrapCollector`, таймер `MirageGenerator`, `SoulEater`, `Necromancer`).
- `src/ui/game_over.rs` — экран результатов матча и рестарт.

## Утилиты (`src/utils/`)
- `src/utils/mod.rs` — чистые математические хелперы (расстояние до отрезка, пересечение кругов).
