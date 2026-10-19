# Deanfinite: независимые графические ресурсы

База изменения: main `aaf5ddb56dc586c3087fce1bacae59f553c37bcf`.
Графика встроена во frontend. Product Deanfinite, identifier
`io.github.deanwell15.deanfinite`, пути настроек и цен не меняются.

## Инвентаризация до изменения

| Группа | Где использовалась | Обязательность и происхождение |
|---|---|---|
| `public_assets/icons/<item_id>.png` | `LootFeed.vue` (1 место), `ItemsHistory.vue` (3), `ExtraEffectsOverlay.vue` (2) через `getIcon` | Для узнаваемости предмета; данные и расчёты от картинки не зависят. В репозитории набора нет. Условия внешнего архива неизвестны; возможны игровые ресурсы. |
| `public_assets/piganim/pig_0.png` … `pig_119.png` | `PiggyWidget.vue`: 4 уровня × 30 кадров, 30 FPS | Декоративная свинка, не условие расчёта. Тот же неподтверждённый архив. |
| `public_assets/piganim/coin/coin_0.png` … `coin_121.png` | `PiggyWidget.vue`, частицы `ExtraEffectsOverlay.vue` | 122 декоративных кадра. Тот же неподтверждённый архив. |
| Прежние PNG/ICO/ICNS в `src-tauri/icons/` | Конфигурация bundle, tray в `lib.rs`, favicon `public/app-icon.png` | Нужны build/tray; наследованные Tauri-подобные изображения заменены собственными. Неподключённые Square/Store удалены. |
| Vite/Vue SVG | `public/vite.svg`, `src/assets/vue.svg` | Не использовались, удалены. |
| Inter | Google CSS из `index.html`, Nord preset `themeStore.ts` | Декоративный шрифт; заменён системным стеком. Шрифт не копировался. |
| Material Symbols Rounded | Google CSS из `index.html`; единственный span `DraggableWidget.vue`, open_in_new/close_fullscreen | Кнопка окна заменена встроенным SVG, без icon font. |
| Остальные значки и эффекты | Unicode/emoji, CSS, inline SVG в компонентах | Локальная отрисовка/системные шрифты; внешние графические файлы не требуются. |

Прежняя цепочка: App → assetUpdater → check_assets_update (GitHub release
metadata) → download_assets (ZIP) → public_assets + assets_version.json →
get_assets_path → convertFileSrc → изображение. App скрывал основные виджеты
до готовности ресурсов **и** одноразового LoadingDone от backend.
Даже декоративные файлы фактически блокировали показ базового UI.

Archive не скачивался для проверки; происхождение отдельных его файлов нельзя
достоверно разделить на авторские и игровые. MIT исходников этого не разрешает.
Решения по лицензиям перечислены в [ASSET_LICENSES.md](../ASSET_LICENSES.md).

## Новая архитектура

`src/utils/assets.ts` сохраняет синхронные интерфейсы `getIcon(id)` и
`getAnim(path)`. Любой ID возвращает item.svg; coin/* — coin.svg, остальные
кадры — effect.svg. ID не интерпретируется как путь. Это **нейтральные заглушки**:
индивидуальной предметной графики и исходных покадровых анимаций больше нет.
Циклы времени/движения эффектов сохранены, но кадры показывают один эмблемный
рисунок. Новые собственные анимации можно добавить позже без изменения расчётов.

Vite `?url&no-inline` включает SVG в собранный frontend с хешированными именами.
Tauri обслуживает их своим внутренним frontend protocol, как JS/CSS. Файловые
пути AppData, ZIP, metadata, IPC и удалённые fallback не используются.
Каждое окно получает тот же bundled набор независимо от main.

`AssetImage.vue`: ошибочный src → локальный item.svg → CSS-символ ◇, если и
fallback повреждён. Повторных попыток после terminal fallback нет. Смена src
сбрасывает ошибку. Атрибуты, размеры и стили переходят на корневой элемент.
Названия, количество, цены и ID поступают прежним путём и не заменяются картинкой.

App сразу отображает виджеты. LoadingStart/Done управляют только сообщением о
чтении игрового лога; сообщение не перехватывает мышь. Без лога оно может
оставаться видимым, но не скрывает UI. Таймеры и загрузка настроек не ждут графику.

Удалены assets.rs, assetUpdater.ts и три asset-команды. Убраны zip-extract и
неиспользуемые транзитивные зависимости, без обновления оставшихся версий.
`assetProtocol.enable=false`, `scope=[]`; Cargo feature protocol-asset убрана.
Доступ к произвольным файлам через этот протокол не нужен. Новых capabilities
или сетевых разрешений нет; production CSP пока прежняя (`null`).
Пользовательские старые public_assets/metadata/ZIP остаются на диске нетронутыми.

## Автоматические проверки

```shell
npm ci
npm run test:assets
npm run build
```

Нужен уже установленный Chrome/Chromium; нестандартный путь задаётся CHROME_BIN.
Загрузка браузеров/исполняемых файлов тестами не выполняется. Node 24 из CI.
В Windows CI тест запускается перед сборкой MSI/NSIS; используется Chrome runner.

`tests/assets.test.mjs` запускает Vite и настоящий headless Chromium, но
**Tauri IPC/window API подменены** штатными mocks. Тесты выполняют реальный App,
Pinia, обработчик LootDrop и компоненты:

1. Чистый browser context без cache/localStorage: виджеты видимы ещё без LoadingDone.
2. Повторный запуск с тем же origin/storage: frontend settings сохранены.
3. Offline boundary: все нелокальные HTTP(S) запросы блокируются CDP, CSP тестовой
   страницы запрещает внешние ресурсы; попытка внешнего запроса проваливает тест.
4. 404 для bundled SVG: видимый CSS fallback, текст/числа остаются.
5. Повреждённое SVG с HTTP 200: тот же fallback, без зависания запуска.
6. `widgetOnly=lootFeedSession`: отдельный frontend/Pinia/context, лут и числа
   отображаются без main readiness. Нативное создание/возврат окна не эмулируется.

Во всех сценариях проверяются название, x2 и [20.0] для одного fixture-event;
формулы не исправляются и полный экономический аудит этим не подменяется.
Также проверены повреждённый отдельный PNG и восстановление при смене src,
неизвестный ID, отсутствие legacy IPC/URL и закрытый asset protocol.
Тестовые страницы не входят в production entry/bundle; fault injection только в
тестовом Vite server. Это проверка UI, а не полной установки Windows.

## Ручная проверка Windows

1. Actions → Deanfinite Windows test build → Run workflow →
   `feature/deanfinite-local-assets` (или дождаться PR run). Проверить зелёный
   результат именно нужного commit и BUILD_INFO/SHA256SUMS артефакта.
2. В тестовой VM/новой Windows-учётной записи с установленным WebView2 запустить
   один формат (MSI **или** NSIS **или** standalone). Не удалять рабочие данные
   ради теста. При отсутствии игры UI всё равно должен показать виджеты/settings.
3. До первого запуска отключить сеть. Welcome, F8, темы, таймеры и таблицы должны
   работать. Отсутствие свежих цен допустимо; значения без цен могут быть нулевыми.
   Offline-установка WebView2 и новый источник цен в задачу не входят.
4. На том же профиле закрыть/повторно открыть из другого CWD: тема/layout сохранены,
   нет проверки/скачивания ресурсов. Identifier и каталоги прежние Deanfinite.
5. С игровым логом проверить session/map loot, расходы, инвентарь и окно истории:
   общая иконка, читаемые названия/количества/значения, нейтральный эффект/монета.
   Известные дефекты повторного loot/налога не считать исправленными этим PR.
6. Открепить loot/stat widget, проверить данные при отсутствии сети; закрыть/вернуть
   окно. Отдельно записать существующие проблемы синхронизации окон, если есть.
7. Запустить с профилем, где уже есть старые public_assets/assets_version.json/ZIP:
   заранее записать хеши/время файлов. После запуска они неизменны и не используются.
   Повторить с пустой папкой ресурсов; вручную удалять рабочие файлы не требуется.
8. Для fault injection использовать npm run test:assets: 404/битые bundled SVG
   эмулируются без модификации установленного EXE. Не подменять подписываемый бинарник.
9. В WebView2 DevTools/сетевой трассировке проверить все окна и эффекты: нет запросов
   к upstream releases, assets.zip, fonts.googleapis.com/fonts.gstatic.com. Цены
   могут обращаться к прежнему IP. Не путать фоновый трафик Windows/WebView с app.
10. Проверить новую D-иконку EXE/tray/установщика. Кэш значков Windows может требовать
    нового пути тестовой установки; не менять identity ради обновления иконки.

Windows GUI/WebView2/установщики требуют выполнения этих шагов пользователем;
браузерный smoke-test не доказывает их успех.

## Оставшаяся сеть

- Прежний PricingManager: `http://115.231.176.101:8080/get?season_id=N`, override
  `TLI_PRICE_API_URL`, локальный prices_cache.json, 10-секундный timeout — без изменений.
  Принадлежность IP автору не установлена. Независимость **цен** ещё не достигнута.
- WebView2 bootstrap/runtime/update может обращаться к Microsoft. Standalone
  требует установленного runtime и не является полностью portable/offline пакетом.
- GitHub/npm/Cargo, WiX/NSIS tools нужны сборке; schema URL в JSON не runtime fetch.
- Ссылки на оригинальный проект в README/credits — атрибуция, не автообновление.

Следующая отдельная задача — PriceProvider/локальный import/cache и проверка
лицензий/контрактов кандидатов источников; TLIDB/TITrack не подтверждены как рынки.

## Изменённые файлы

```text
.github/workflows/windows-test-build.yml
README.md
docs/deanfinite-branding.md
docs/independence-plan.md
docs/windows-test-build.md
index.html
package.json
public/app-icon.png
public/vite.svg
src-tauri/Cargo.lock
src-tauri/Cargo.toml
src-tauri/icons/128x128.png
src-tauri/icons/128x128@2x.png
src-tauri/icons/32x32.png
src-tauri/icons/Square107x107Logo.png
src-tauri/icons/Square142x142Logo.png
src-tauri/icons/Square150x150Logo.png
src-tauri/icons/Square284x284Logo.png
src-tauri/icons/Square30x30Logo.png
src-tauri/icons/Square310x310Logo.png
src-tauri/icons/Square44x44Logo.png
src-tauri/icons/Square71x71Logo.png
src-tauri/icons/Square89x89Logo.png
src-tauri/icons/StoreLogo.png
src-tauri/icons/icon.icns
src-tauri/icons/icon.ico
src-tauri/icons/icon.png
src-tauri/src/assets.rs
src-tauri/src/lib.rs
src-tauri/tauri.conf.json
src/App.vue
src/assets/vue.svg
src/components/DraggableWidget.vue
src/components/ExtraEffectsOverlay.vue
src/components/ItemsHistory.vue
src/components/LootFeed.vue
src/components/PiggyWidget.vue
src/stores/themeStore.ts
src/utils/assetUpdater.ts
src/utils/assets.ts
ASSET_LICENSES.md
docs/local-assets.md
src/assets/local/app.svg
src/assets/local/coin.svg
src/assets/local/effect.svg
src/assets/local/item.svg
src/components/AssetImage.vue
tests/assets-harness.html
tests/assets-harness.ts
tests/assets.test.mjs
tests/tsconfig.json
tools/generate-app-icons.py
```
