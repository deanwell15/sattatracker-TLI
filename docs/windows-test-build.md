# Deanfinite: тестовая Windows-сборка

Используется прежний [windows-test-build.yml](../.github/workflows/windows-test-build.yml)
в **deanwell15/sattatracker-TLI**. Новый workflow не создавался. CI уже включён в
main и для предыдущей identity успешно прошёл Windows-сборку; пользователь
подтвердил её запуск. Это не подтверждает новую Deanfinite-сборку.

## Что сохраняется и что изменилось

- Windows runner `windows-2022`, Node 24.19.0, Rust 1.99.1, MSVC x64.
- `workflow_dispatch`, `pull_request` в main, TypeScript, frontend через существующий
  Tauri beforeBuildCommand, Cargo `--locked`, MSI и NSIS.
- Официальные actions закреплены SHA; `contents: read`, без persisted credentials,
  release/signing secrets, push или GitHub Release.
- После сборки выполняются четыре unit-теста `settings::tests` в release-профиле:
  сохранение/перезагрузка, чистая конфигурация, невалидный JSON и ошибка записи.
- EXE теперь **Deanfinite.exe**, installer prefix **Deanfinite**. CI проверяет
  регистр через `-cne`/`-cnotlike`; старое имя EXE не ищется.
- Artifact `deanfinite-windows-x64-<run_id>-<run_attempt>` хранится 14 дней.
- SHA256SUMS, BUILD_INFO, LICENSE и инструкции сохраняются. CLI Tauri приходит
  из npm lock; bundler получает свои WiX/NSIS tools. Графический assets.zip не
  включается, приложение в CI не запускается (выполняются только settings tests).

## Запуск

PR из `feature/deanfinite-branding` в main **нашего форка** запускает проверку.
Существующий workflow уже есть в default branch, поэтому ручной запуск также
можно сделать с feature-ветки:

1. Actions → **Windows test build** (после обновления default branch название
   будет **Deanfinite Windows test build**) → Run workflow.
2. В `Use workflow from` выбрать **feature/deanfinite-branding**.
3. Запустить и дождаться результата именно нового commit/run.

Через авторизованный GitHub CLI:

```shell
gh workflow run windows-test-build.yml --repo deanwell15/sattatracker-TLI --ref feature/deanfinite-branding
```

Для PR checkout обычно соответствует merge commit; его точный SHA записан в
BUILD_INFO. Не выдавать успешную сборку старой identity за результат нового PR.

## Получение приложения

После зелёного run: Summary → Artifacts → скачать ZIP:

```text
deanfinite-windows-x64-<run_id>-<run_attempt>.zip
├── installers/msi/Deanfinite*.msi
├── installers/nsis/Deanfinite*.exe
├── standalone/Deanfinite.exe
├── BUILD_INFO.json
├── SHA256SUMS.txt
├── RUNNING.txt
└── LICENSE
```

```shell
gh run download <run_id> --repo deanwell15/sattatracker-TLI
```

```powershell
Get-FileHash .\standalone\Deanfinite.exe -Algorithm SHA256
```

Сверить с SHA256SUMS. Это контроль целостности, не Authenticode-подпись.
BUILD_INFO содержит source commit, version, productName, identifier, toolchain,
runner image и хеши lock-файлов. Если любой установщик/EXE отсутствует или имеет
неверное имя, job должен завершиться ошибкой; артефакт не загружается как успешный.

## Standalone и установка

Standalone — запуск без установки, не полностью portable/offline-пакет:
нужен Microsoft Edge WebView2 Runtime, сохраняется requireAdministrator, данные
пишутся в отдельные Deanfinite AppConfig/LocalAppData, цены/ресурсы пока сетевые.
Не запускать две Deanfinite-копии параллельно и не ставить MSI/NSIS одного продукта
рядом друг с другом. Совместимость с первоначальной установкой и тест удаления:
[deanfinite-branding.md](deanfinite-branding.md).

Прежний [release.yml](../.github/workflows/release.yml) не изменён и реагирует на
теги v*. Для тестовой сборки не создавать такие теги: новый run использовать
через PR или workflow_dispatch. Merge и Release в этой задаче не выполняются.
