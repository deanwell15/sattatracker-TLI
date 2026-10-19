# Deanfinite

**Deanfinite — Torchlight: Infinite Loot Tracker**

Независимый форк [SattaTracker](https://github.com/sattatrapy/sattatracker-TLI-Tauri-public),
сопровождаемый `deanwell15`. Первоначальный автор исходного проекта — `sattatrapy`.
Репозиторий форка пока сохраняет имя [deanwell15/sattatracker-TLI](https://github.com/deanwell15/sattatracker-TLI).

Прозрачный оверлей на Tauri 2/Rust и Vue 3: лут, расходы, прибыль, таймеры,
снимок инвентаря, темы и перемещаемые виджеты. Игровые события поступают из лога.
Поиск пути к логу использует также сведения о процессах; это не обещание полного
отсутствия обращений библиотеки sysinfo к памяти процессов.

## Управление

| Клавиша | Действие |
|---|---|
| F7 | Скрыть/показать оверлей |
| F8 | Открыть меню виджетов и настройки |

В игре включить логирование через ESC, открыть инвентарь и выполнить сортировку.
Графические изменения не меняют parser, цены и расчёты. Известные ошибки
статистики из аудита остаются отдельной задачей.

## Сборка и проверка

Рабочий Windows CI уже находится в main. Для Deanfinite используется тот же
[workflow](.github/workflows/windows-test-build.yml), адаптированный к новой identity.
Успех прежней сборки не подтверждает новую версию: проверить run этого PR/коммита.

В CI: Node 24.19.0, Rust 1.99.1, Windows 2022/MSVC, существующие npm/Cargo locks.
Для локальной Windows-сборки нужны также Visual Studio C++ Build Tools и Windows SDK.

```shell
npm ci
npm run test:assets # requires installed Chrome/Chromium
npm run tauri -- dev
npm run tauri -- build --ci --target x86_64-pc-windows-msvc --bundles nsis,msi -- --locked
```

Ожидаемый EXE — **Deanfinite.exe**. Standalone EXE требует Microsoft Edge WebView2
Runtime и пока не является полностью portable/offline-приложением. Release
по-прежнему использует `requireAdministrator`; этот манифест не изменялся.

Инструкции: [Windows CI](docs/windows-test-build.md),
[изоляция данных и Windows smoke-test](docs/deanfinite-branding.md).

## Собственные данные Deanfinite

Identifier: **io.github.deanwell15.deanfinite**. Для выполняющего приложение
Windows-пользователя используются:

| Данные | Путь |
|---|---|
| Backend flags | `%APPDATA%\io.github.deanwell15.deanfinite\config.json` |
| Цены | `%LOCALAPPDATA%\io.github.deanwell15.deanfinite\prices_cache.json` |
| Графика | Встроена в frontend; внешняя папка и кэш ресурсов не требуются |
| WebView2 / localStorage | Собственный профиль под `%LOCALAPPDATA%\io.github.deanwell15.deanfinite` |

Настройки не читаются рядом с EXE или из текущего рабочего каталога. Каталоги
создаются автоматически; формат двух backend flags сохранён. Сохранение без
запущенной игры поддерживается. Ошибка записи отображается в окне настроек.
Повреждённый config не перезаписывается при загрузке: ошибка логируется,
используются defaults до явного сохранения пользователем.

Первый запуск использует чистые настройки. Автоимпорта из SattaTracker нет,
старые файлы не удаляются. Ключи localStorage остаются прежними, но профиль
WebView2 отделён новым identifier. Ручной импорт будет отдельной задачей.

## Текущие внешние зависимости

Графика теперь независима от upstream: собственные нейтральные SVG встроены
в приложение, иконка D — собственная, шрифты системные. Индивидуальные игровые
картинки и исходные кадры свинки/монет заменены заглушками. Названия и значения
предметов сохранены. Нет загрузки GitHub assets.zip, внешних шрифтов или чтения
старого public_assets; существующие пользовательские файлы не удаляются.
[Инвентаризация, архитектура, тесты и Windows checklist](docs/local-assets.md),
[лицензии графики](ASSET_LICENSES.md).

Остаются цены `http://115.231.176.101:8080/get?season_id=N`, override
`TLI_PRICE_API_URL` и прежний формат `{ "<item_id>": { "price": <number> } }`.
Оператор сервера не подтверждён. UI и bundled графика работают без интернета;
без сети/кэша свежие цены недоступны. Автообновления самого EXE нет.
Microsoft WebView2 остаётся обязательной внешней платформой.

Дальнейшая замена: [план независимости](docs/independence-plan.md).

## Лицензия и авторство

[MIT](LICENSE), исходное уведомление **Copyright (c) 2026 sattatrapy** сохранено.
Сопровождение форка: `deanwell15`; авторство первоначального исходного кода не
присваивается. Лицензия исходников сама по себе не устанавливает права на
сторонние игровые изображения, внешний assets.zip и шрифты.
