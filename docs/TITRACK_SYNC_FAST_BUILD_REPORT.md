# Deanfinite PR #4: синхронизация TITrack и Fast Build

Дата: 20 октября 2026. Ветка: `feature/deanfinite-pricing`.
PR: https://github.com/deanwell15/sattatracker-TLI/pull/4. Merge и Release не выполняются.

## 1. Причина ошибки

В `TiTrackPriceProvider::decode_aggregates()` (`src-tauri/src/pricing.rs`) выполнялась
проверка `unique_devices < 3 || submission_count < unique_devices` с возвратом `Err`.
Вызывающий `fetch_table()` валидирует каждую страницу и предварительный запрос
верхней границы. Достаточно одной такой строки, чтобы отменить весь batch.
`PricingManager::refresh_at()` правильно сохранял предыдущий кэш при ошибке, но
на первой загрузке предыдущих цен нет: отсюда Cloud cache 0 и never.

Проверенный источник TITrack: commit
[7798e02](https://github.com/astockman99/TITrack/tree/7798e02db8cfc5fb3b5eb927e3600fef99815b5e).
В [001_initial_schema.sql, aggregate_prices(), строки 135–190](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/supabase/migrations/001_initial_schema.sql#L135)
`submission_count` — COUNT(*) подходящих исходных записей, `unique_devices` —
COUNT(DISTINCT device_id) среди неотмеченных устройств за последние 24 часа.
SQL этой версии проверяет минимум три устройства перед upsert. Но это логика
сервера данной версии, а не CHECK-ограничение всех строк таблицы или контракт,
который обязан дополнительно исполнять сторонний клиент.

В [CloudPrice, строки 22–32](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/src/titrack/sync/client.py#L22)
оба счётчика Optional; `fetch_prices_delta()`, строки 189–250, использует `row.get`
и не отвергает серверные агрегаты по этому порогу. Состояние развёрнутой БД нельзя
выводить только из исторической миграции. Сообщение ошибки Windows доказывает
срабатывание одного из двух условий; конкретные значения проблемной строки
пользователь не предоставил, их здесь не придумываем.

## 2. Исправление

Клиентский quorum veto удалён. Корректные серверные median принимаются независимо
от малого числа наблюдений; отсутствующие/null счётчики разрешены. При наличии
значения счётчик остаётся неотрицательным целым `u32`. Это не заявление о высокой
достоверности конкретной цены: TITrack остаётся источником агрегированных оценок.

Сохраняются строгие проверки сезона 1601, числового ID, конечных неотрицательных
цен и percentile, FE=1, RFC3339 timestamp, размера ответа, числа строк/страниц,
монотонности ID, временных границ, HTTP статуса, TLS и запрета redirects.
Некорректный JSON, транспорт или пагинация отменяют ВСЮ синхронизацию.
Атомарная запись цен и cursor выполняется только после полного успеха.

`SyncStatistics` хранит `received`, `accepted`, `skipped`, `skip_reasons`,
`low_observation_rows`. Счётчики относятся к страницам данных, предварительная
строка watermark повторно не учитывается. Сейчас дополнительной политики отсева
нет: skipped=0 и skip_reasons={}; малое число наблюдений только диагностируется.
Структурно невалидная строка вызывает ошибку batch, а не скрытый skip.
Статистика последней успешной загрузки сохраняется вместе с кэшем, передаётся
в `PricingSnapshot.last_sync`, отображается в Market prices и пишется в безопасный
лог. Ошибка следующего запроса не подменяет статистику успешной загрузки.

HTTP остаётся read-only; 300 секунд, delta и inclusive timestamp boundary,
периодическая полная синхронизация, backoff/Retry-After и offline cache сохранены.
Ключ, HTTP headers/body, игровые логи, инвентарь и аукционные запросы не выводятся
и не отправляются. GameStateMachine, парсер, учёт предметов и Min Drop Value не менялись.

## 3. Регрессионные проверки

- `mixed_observation_counts_are_metadata_not_a_batch_veto`: корректные строки +
  строка с одним устройством + null/отсутствующие счётчики; низкие счётчики есть
  даже в watermark; три цены попадают в RAM, JSON-кэш и offline restart.
- `real_http_pages_populate_cloud_cache_with_2501_prices_and_offline_restart`:
  настоящий reqwest по loopback HTTP, три страницы + EOF, 2501 цена, Cloud cache
  2501, перезапуск из кэша. Это mock HTTP, не live Supabase.
- `malformed_later_page_does_not_commit_prices_statistics_or_cursor`: испорченный
  JSON после корректной страницы не меняет байты старого кэша, цены и статистику.
- Браузерный `assets-harness.ts`: реальный компонент Market prices показывает
  Cloud cache 2501, 2501 accepted, 0 skipped; исчезает предыдущая ошибка; количество
  предметов, latestDrop и результат переоценки не меняются.
- `fast-build.test.mjs`: PE32+ x64, запрет MSI/NSIS/icons в Fast, неизменность
  контракта ресурсов и обнаружение изменённого hash.
- Существующие проверки offline fallback, переоценки без LootDrop, backoff,
  ошибок транспорта, цен/сезонов, оригинальных анимаций и icon protocol сохранены.

Локально: 36 Rust passed; отдельный live-тест ignored. TypeScript + Vite проходят;
9 Pinia-сценариев; 6 Chromium-сценариев; 3 проверки установочных EXE; 3 новые
проверки Fast; 8 Python self-tests; все 3088 исходных изображений и 243 анимационных
файла production frontend проверены. YAML проверен по GitHub Actions JSON schema.
Linux-проверка Rust выполняется прежним модульным harness без GTK/WebKit;
она не заменяет native Windows CI.

## 4. Fast Build

Tauri CLI поддерживает `--no-bundle`; обход CLI не потребовался:

```powershell
npm run tauri -- build --ci --target x86_64-pc-windows-msvc --no-bundle -- --locked
```

`beforeBuildCommand` по-прежнему собирает frontend и проверяет TypeScript.
Release-профиль Fast использует LTO=false, codegen-units=16 только через окружение
CI. Это ускоряет компиляцию без изменения логики и ресурсов. Full сохраняет
LTO=true/codegen-units=1 из production-профиля. Fast и Full кэши разделены.

Fast artifact содержит ровно восемь файлов:
`Deanfinite.exe`, `BUILD_INFO.json`, `RESOURCE_COMPATIBILITY.json`,
`CHECK_COMPATIBILITY.ps1`, `RUNNING.txt`, `LICENSE`, `ASSET_LICENSES.md`,
`SHA256SUMS.txt`. MSI, NSIS, папки icons/, standalone/ и отчётов в нём нет.
Анимации свинки и монет остаются встроены в frontend EXE. Повторная передача этих
байтов вместе с EXE неизбежна; внешние 2845 иконок в архив не копируются.

Это EXE для замены в существующем комплекте, **не полностью автономное приложение**.
Нужны совместимая папка icons/ и Microsoft Edge WebView2 Runtime.

## 5. Совместимость ресурсов

`assets/standalone-resource-contract.json` фиксирует существующий выпуск 2845
иконок: имена + SHA-256 исходных байтов. Manifest ресурсов и сами картинки не меняются.
`iconContract()` строит стабильный digest отсортированных ASCII-имён и hash,
`verifyIcons()` проверяет все ресурсы рядом с собранным EXE. В артефакт передаётся
только маленький контракт, не файлы изображений.

`CHECK_COMPATIBILITY.ps1` на компьютере пользователя проверяет ту же структуру
и каждый hash существующего kit; CI проверяет и успешный случай, и отказ при
неверном digest. `verifyPeX64()` проверяет MZ, PE, machine=0x8664 и PE32+.
Product name Deanfinite, identifier io.github.deanwell15.deanfinite, AppData,
manifest, custom icon protocol и схема размещения ресурсов не изменены.

## 6. Запуск GitHub Actions

Actions → **Deanfinite Windows test build** → Run workflow → ветка
`feature/deanfinite-pricing` → `build_mode=fast` → Run workflow.
Если форма ещё использует старую версию из main без поля режима, ручной запуск
без input тоже выбирает Fast в новой версии workflow выбранной ветки.

Через свой авторизованный GitHub CLI можно явно задать режим:

```powershell
gh workflow run windows-test-build.yml --repo deanwell15/sattatracker-TLI --ref feature/deanfinite-pricing -f build_mode=fast
```

Full: `-f build_mode=full`. Дополнительная live-диагностика для ручного Full:
`-f build_mode=full -F live_prices=true`.

Обновление workflow: PR в main автоматически запускает только Fast.
Одна стабильная job `build-windows` выполняет все общие тесты и выбранную упаковку;
Full доступен вручную перед merge для текущей ветки. Матрица удалена.
Фактическая защита main отключена; для запрета merge без проверок требуется
настройка GitHub — см. [Windows CI и branch protection](windows-test-build.md).
Результаты: Artifacts → `deanfinite-windows-fast-x64-<run>-<attempt>` либо
`deanfinite-windows-full-x64-<run>-<attempt>`. Хранение 14 дней, compression-level=0.

## 7. Установка Fast EXE

1. Полностью закрыть Deanfinite.
2. Сохранить резервную копию всего текущего standalone kit.
3. Распаковать Fast artifact отдельно и проверить существующие ресурсы:
   `powershell -NoProfile -File .\CHECK_COMPATIBILITY.ps1 -KitRoot 'C:\path\to\standalone'`.
4. Заменить только Deanfinite.exe внутри прежнего standalone kit.
5. Не удалять папку icons/, оставить её рядом с EXE.
6. Запустить приложение из прежнего каталога; проверить цены и анимации.

При несовпадении контракта использовать совпадающий Full Build kit. Не запускать
одновременно старую и новую копии. Настройки и кэш остаются в прежних путях.

## 8. Full Build и кэш

Full сохраняет EXE, standalone kit с icons/, MSI, NSIS, лицензии, отчёты,
контрольные суммы и проверки всех icon hashes в release/MSI CAB/NSIS, а также
сравнение встроенных EXE с учётом только штатной Tauri bundle marker patch.

npm cache включён через setup-node; `npm ci` продолжает исполняться по lock-файлу.
Cargo cache сохраняет registry/git и target. Ключ: Windows 2022, Rust 1.99.1,
x64, режим оптимизации, Cargo.lock и hash исходников/конфигурации. Restore prefix
того же lock/profile позволяет повторно использовать зависимости между коммитами.
Cargo всё равно сверяет fingerprints; локальные изменения не игнорируются.

GitHub hosted runners одноразовые. Кэш не гарантирован: ограничение квоты репозитория
(обычно 10 GiB по умолчанию, зависит от настроек/плана), eviction, отсутствие доступа
к кэшу другого ref, обновления runner image/SDK, toolchain/profile/lock вызывают
промах или пересборку. PR cache привязан к merge ref; ручной запуск ветки может
не получить его. node_modules не кэшируется. Full target с LTO крупный; download/
upload кэша сам занимает время. Гарантировать ускорение до повторного запуска нельзя.
Проверки не отключены, pull_request_target/secrets/внешние shell-скрипты не добавлялись.

## 9. Live и автоматические результаты — отдельно

**Подтверждено пользователем с Windows:** Supabase HTTP 200, публичный publishable
key работает, aggregated_prices доступна; SS14/1601, config_base_id=350714,
median=388, updated_at=2026-10-20T08:45:00.442384+00:00.
Это проверка API пользователем, не proof успешной работы нового EXE.

**Codex:** прежняя попытка остановилась на CONNECT proxy 403; HTTP статус самого
Supabase не получен. Новые live-запросы из этого Linux окружения не выполнялись.

**Windows CI:** обязательные offline/mock tests отделены от ignored live-теста
`live_titrack_populates_isolated_cloud_cache`. В текущем workflow
он отдельно вызывается только в ручном Full с live_prices=true. Ранее отдельный
запуск Full на Windows runner подтвердил загрузку 2073 реальных цен; это
исторический результат, не новая live-проверка при каждом push.
Создаётся изолированный временный PricingManager cache, без AppData и игровых данных.
Результат сохраняется в `LIVE_TITRACK_RESULT.json` Full artifact. Live ошибка даёт
предупреждение и `live_verified=false`, не отменяет корректность сборки/оффлайн-тестов.
Зелёный workflow сам по себе **не доказывает** live: нужен этот результат с
live_verified=true и cloud_count>0. GUI/game smoke по-прежнему требует пользователя.

## 10. Измерения и окончательный статус

Размеры и время новой Fast/Full сборок заполняются по фактическим Actions artifacts
и job timestamps в итоговом отчёте и PR после выполнения workflow. На момент записи
исходного документа эти новые результаты ещё не объявляются успешными.

Предыдущий Full commit 22a99f9: run
[39399701737](https://github.com/deanwell15/sattatracker-TLI/actions/runs/39399701737),
ZIP 559412271 bytes (533.5 MiB), 40 native Rust tests passed. Этот прошлый результат
не подтверждает текущие изменения. Полный итоговый отчёт сохраняется отдельным
файлом с фактическими ссылками, размерами, временем и результатами текущего запуска.
