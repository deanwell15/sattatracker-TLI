# Deanfinite: тестовая Windows-сборка

Workflow [.github/workflows/windows-test-build.yml](../.github/workflows/windows-test-build.yml)
работает в `deanwell15/sattatracker-TLI` на Windows 2022, Node 24.19.0,
Rust 1.99.1, MSVC x64. Исходники приложения, identity, цены и ресурсы не меняются.

## Одна стабильная проверка, два варианта упаковки

- `pull_request` в main запускает только Fast, без матрицы и параллельного Full.
- `workflow_dispatch` выбирает `build_mode=fast` (по умолчанию) или `full`.
- Единственная job/check называется **`build-windows`** при любом режиме.
  Это прежнее имя Full-проверки: ожидание старой Full job не остаётся Pending.
- Внутри этой job всегда выполняются TypeScript, pricing/Pinia tests,
  browser smoke, self-tests диагностического клиента, Fast contract tests,
  frontend build, полный native `cargo test --lib` и проверка lock-файлов.
  У этих шагов нет условия по режиму и нет continue-on-error. Ошибка теста,
  сборки, проверки ресурсов или загрузки артефакта делает check неуспешным.
- Fast/Full — условные шаги сборки и упаковки внутри той же job. Дополнительная
  пустая «успешная» gate job не создаётся. Сам GitHub запрещает merge по required
  check только после включения branch protection/ruleset — см. ниже.

Сохранены Cargo/npm cache, раздельные ключи Fast/Full, существующие locks,
SHA-pinned actions, contents:read, checkout без persisted credentials.
Используются обычный pull_request и workflow_dispatch; pull_request_target,
release/signing secrets и новые внешние скрипты не добавлены.
Старые artifacts не удаляются: прежняя retention policy 14 дней сохранена.

## Fast Build

Tauri `build --ci --target x86_64-pc-windows-msvc --no-bundle -- --locked`.
Анимации остаются внутри EXE; иконки остаются внешним ресурсом.

Artifact `deanfinite-windows-fast-x64-<run>-<attempt>` содержит ровно:
Deanfinite.exe, BUILD_INFO.json, RESOURCE_COMPATIBILITY.json,
CHECK_COMPATIBILITY.ps1, RUNNING.txt, LICENSE, ASSET_LICENSES.md, SHA256SUMS.txt.
Без MSI/NSIS, icons/ и полного standalone kit. PE32+ x64 и hash-контракт
2845 иконок проверяются перед upload. Положительная и отрицательная проверки
совместимости выполняются; installer payload checks относятся к Full.

Это замена EXE в существующем комплекте, не полностью автономное приложение.
Закрыть Deanfinite, сохранить резервную копию kit, проверить
`CHECK_COMPATIBILITY.ps1 -KitRoot <путь>`, заменить только EXE, оставить icons/.
Нужен установленный Microsoft Edge WebView2 Runtime. AppData не меняется.

## Ручной Full перед merge

Actions → **Deanfinite Windows test build** → Run workflow →
`feature/deanfinite-pricing` → `build_mode=full` → Run workflow.
Проверить, что run соответствует текущему HEAD PR; после следующего push
предыдущий Full уже не подтверждает новую ревизию.

```shell
gh workflow run windows-test-build.yml --repo deanwell15/sattatracker-TLI --ref feature/deanfinite-pricing -f build_mode=full
```

Для ручного Fast: `-f build_mode=fast`. Если форма из main пока не показывает
новые inputs, ручной запуск без input на текущей ветке выбирает Fast; Full
можно явно задать приведённой командой GitHub CLI.

Full сохраняет frontend/EXE, standalone/Deanfinite.exe + icons/, MSI, NSIS,
проверки всех исходных hashes в release/MSI CAB/NSIS/standalone, сравнение
встроенных EXE с точной Tauri bundle-marker patch, лицензии, отчёты,
BUILD_INFO и SHA256SUMS. Artifact: `deanfinite-windows-full-x64-<run>-<attempt>`.
Отдельная read-only live-диагностика TITrack запускается только в ручном Full
с `live_prices=true` (`-F live_prices=true`); она не заменяет обязательные тесты.

После зелёного run: Summary → Artifacts → скачать ZIP. Это Actions Artifacts,
не GitHub Release. Контрольные суммы подтверждают целостность, не Authenticode.
Кэш может быть вытеснен; PR merge-ref cache не гарантированно доступен ручному
запуску feature branch. Full cache и Fast cache намеренно разделены.

## Branch protection: фактическое состояние и необходимая настройка

Проверка API 20 октября 2026:
`GET /repos/deanwell15/sattatracker-TLI/branches/main` вернул
`protected=false`, `protection.enabled=false`,
`required_status_checks.enforcement_level=off`, `contexts=[]`, `checks=[]`.
`GET /repos/deanwell15/sattatracker-TLI/rulesets` вернул `[]`.
Это означает, что запрет merge без зелёных проверок сейчас НЕ настроен.
Прямой административный endpoint `/branches/main/protection` возвращает
интеграции 403 Resource not accessible by integration. Настройки GitHub не менялись.

Владелец репозитория должен открыть
[Settings → Branches](https://github.com/deanwell15/sattatracker-TLI/settings/branches)
и создать/изменить правило для `main`:

1. Включить **Require a pull request before merging**.
2. Включить **Require status checks to pass before merging**.
3. Выбрать check **`build-windows`**, источник **GitHub Actions**.
4. Включить **Require branches to be up to date before merging**.
5. Включить **Do not allow bypassing the above settings** для применения к
   администраторам/ролям обхода, если запрет должен действовать для всех.
6. Если ранее вручную добавлен `build-windows (fast)` или `build-windows (full)`,
   заменить его на **`build-windows`**: этих дополнительных checks больше нет.

Альтернатива — активный branch ruleset для main с теми же требованиями и без
bypass actors. Не требовать имя всего workflow вместо имени check. Прежнее
имя `build-windows` сохранено, его менять не нужно.

После настройки убедиться в PR: `build-windows` отмечен Required; пока он Pending,
Failure или Cancelled, merge недоступен. YAML сам по себе не может включить
серверную защиту ветки. Ручной Full — дополнительная проверка перед merge,
а автоматический required check всегда выполняет все общие тесты в Fast.
Ручной Full для HEAD и PR Fast для тестового merge commit — разные git refs;
успех manual run не следует считать заменой свежей PR-проверки.

Прежний release.yml не изменён. Merge, создание тегов и Release не выполняются.
