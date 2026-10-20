# Deanfinite: идентичность, изоляция данных и проверка Windows

## Реализованная идентичность

| Параметр | Значение |
|---|---|
| Product / window title | Deanfinite |
| Описание | Deanfinite — Torchlight: Infinite Loot Tracker |
| Tauri identifier | io.github.deanwell15.deanfinite |
| npm / Cargo package | deanfinite |
| Rust library | deanfinite_lib |
| Cargo explicit binary / default-run | Deanfinite |
| mainBinaryName / Windows EXE | Deanfinite / Deanfinite.exe |
| Publisher установщиков | deanwell15 |
| MSI UpgradeCode | 849ca07a-a9b1-5a63-a3dd-ada5058d0f97 |

UpgradeCode закреплён для будущих обновлений Deanfinite, не менять его между
релизами. Он отличается от прежнего автоматически получаемого UpgradeCode
SattaTracker. Версия 1.2.11 пока сохранена. Labels `main`, динамические
`widget_*`, permissions/capabilities и Windows requireAdministrator не менялись.

Меняются title HTML, builder, welcome, settings и подпись трея. Существующие
иконки впоследствии заменены собственным геометрическим D-комплектом; favicon,
tray и установщики используют его. Подробности: [local-assets.md](local-assets.md).

Исходники: [Tauri config](../src-tauri/tauri.conf.json),
[Cargo](../src-tauri/Cargo.toml), [main](../src-tauri/src/main.rs),
[инициализация и команды](../src-tauri/src/lib.rs),
[сохранение](../src-tauri/src/settings.rs).

## Пути и сохранность данных

Пути формируются Tauri, а не CWD или расположением EXE:

| Resolver / данные | Windows-путь |
|---|---|
| app_config_dir / config.json | `%APPDATA%\io.github.deanwell15.deanfinite\config.json` |
| app_local_data_dir / prices_cache.json | `%LOCALAPPDATA%\io.github.deanwell15.deanfinite\prices_cache.json` |
| Старый public_assets (не используется) | `%LOCALAPPDATA%\io.github.deanwell15.deanfinite\public_assets` |
| Старая metadata / временный ZIP (не используются) | `%LOCALAPPDATA%\io.github.deanwell15.deanfinite\` |
| WebView2 user data directory | `%LOCALAPPDATA%\io.github.deanwell15.deanfinite` |

Это конкретные пути, следующие из конфигурации и source Tauri; наблюдение файлов
на новой Windows-сборке ещё требуется. Значения APPDATA относятся к пользователю,
под которым реально выполняется elevated приложение. Внутренние подпапки профиля
создаёт WebView2, они не задаются вручную приложением.

Tauri 2.10.3 `path/desktop.rs:238–259` дописывает identifier к config/local data.
`manager/webview.rs:505–515` задаёт отдельный data_directory через LocalData и
identifier. Поэтому одинаковые localStorage keys `tracker_config_v2`,
`tracker_theme_v2`, `tli_welcomed_v1` изолированы профилем. Их переименование и
автоматическая миграция не нужны.

Для сравнения: старая identity — `com.fedin.sattatracker`. Её каталоги не читаются
и не изменяются; `config.json` рядом с прежним EXE также не импортируется. Нет
fallback к shared CWD для config или price cache. Существующая конфигурация
оригинального приложения не переносится даже при отсутствии Deanfinite config.

Каталоги создаются при setup. Ошибки разрешения/создания каталога не заменяются
путём оригинального приложения. Ошибки чтения/JSON диагностируются без изменения
файла; явное Save сообщает об ошибке записи и не закрывает settings. Успешный
Save сначала записывает файл, затем обновляет watch state, в том числе без tailer.
Формат `ignore_auction`/`apply_tax` прежний. История сессий не добавлялась.

## Windows installer identity

По стандартным templates bundler Tauri CLI 2.10.1:

- MSI получает `Name=Deanfinite`, `Manufacturer=deanwell15`, свой UpgradeCode и
  отдельную директорию `Deanfinite` внутри ProgramFiles64 для x64.
- NSIS использует product name для Uninstall registry key, product+publisher
  для сохранённого install path и identifier для удаления app data.
- Default NSIS per-user путь — `%LOCALAPPDATA%\Deanfinite`; выбор режима/папки
  пользователем может его изменить. Не выбирать вручную старую папку SattaTracker.
- Ожидаемое имя в «Установленных приложениях» — Deanfinite, процесс — Deanfinite.exe.
- NSIS uninstall с выбранным удалением данных удаляет APPDATA/LOCALAPPDATA только
  для нового identifier. MSI по умолчанию удаляет установленный продукт;
  удаление пользовательских данных не обещается.

Это source-level проверка, не результат запуска установщиков. Окончательно
сверить generated MSI/NSIS и uninstall на Windows. Новые имя, publisher и
UpgradeCode не должны распознавать старый продукт как обновляемую Deanfinite.
Для одного Deanfinite не ставить MSI и NSIS параллельно: это разные способы
установки одного продукта.

Исходники bundler:
[NSIS](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.10.1/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi),
[MSI](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.10.1/crates/tauri-bundler/src/bundle/windows/msi/main.wxs).

## Совместный запуск

Single-instance plugin не добавлен: обе программы могут открываться отдельно и
читать один и тот же лог. Они имеют разные EXE, installer identity и data/profile
directories. Однако обе используют глобальные F7/F8: второй процесс может не
получить эти hotkeys. Регистрация клавиш и layout намеренно не менялись; полную
одновременную работу shortcuts не заявляем.

## Windows smoke-test новой сборки

1. Создать backup/snapshot существующей установки и её данных. Использовать
   тестовую VM/учётную запись; записать старые пути и значения настроек.
2. Скачать artifact нужного PR/run, сверить SHA256SUMS и BUILD_INFO:
   productName Deanfinite, identifier io.github.deanwell15.deanfinite, нужный commit.
3. Убедиться в точном регистре `standalone/Deanfinite.exe` и префиксе Deanfinite у
   MSI/NSIS. В CI это дополнительно проверяется.
4. Установить один формат в отдельный default path. Проверить имя, publisher,
   shortcut target, процесс и самостоятельную запись в «Установленных приложениях».
5. Первый запуск: чистые настройки и welcome. Проверить новые APPDATA/LOCALAPPDATA,
   отсутствие config рядом с EXE и отсутствие изменений старых данных.
6. Изменить backend flags и frontend theme/layout, закрыть и повторно запустить
   Deanfinite из другого рабочего каталога. Проверить сохранение обоих наборов
   настроек и собственного price cache. Проверить Save также без найденного игрового лога.
7. Проверить лог, overlay, F7/F8, инвентарь и графику. Это проверка сохранения
   поведения, не исправления известных ошибок статистики.
8. Запустить обе программы, проверить раздельные данные. Зафиксировать, какая
   программа получила глобальные hotkeys; не считать их конфликт утечкой данных.
9. Удалить Deanfinite, отдельно проверить NSIS опцию удаления собственных данных.
   Существующая установка SattaTracker, её запуск, EXE и данные должны сохраниться.
10. MSI/NSIS и standalone проверять последовательно; будущие upgrade-тесты
    Deanfinite должны сохранять его стабильный UpgradeCode.

## Что намеренно осталось прежним

- Репозиторий `deanwell15/sattatracker-TLI` не переименован.
- SattaTracker и sattatrapy в credits/MIT/upstream URL — сохранённая атрибуция.
- `com.fedin.sattatracker` встречается только в документации сравнения и проверок
  старой установки, не как новый runtime path.
- Автоматические upstream assets/metadata и Google Fonts удалены в графической задаче;
  старые пользовательские файлы оставлены нетронутыми.
- Цена 115.231.176.101:8080 и WebView2 остаются внешними зависимостями.
- Generic localStorage keys сохранены в отдельном WebView-профиле.
- Parser, FSM, bag manager, PricingManager, налог/прибыль и log processing не
  изменены. В tailer меняется только источник пути cache, уже разрешённого при setup.
