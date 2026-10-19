# Тестовая Windows-сборка независимого форка

Workflow: [windows-test-build.yml](../.github/workflows/windows-test-build.yml).
Репозиторий: **deanwell15/sattatracker-TLI**, целевая ветка PR: **main этого форка**.

Workflow подготовлен для выполнения на Windows. До успешного GitHub Actions run
наличие workflow и локальные проверки не означают успешную Windows-сборку.
GUI, игровые логи, оверлей и установщики требуют отдельного Windows smoke-test.

## Что выполняется

1. Checkout ревизии события без сохранения GitHub credentials в git config.
2. Node.js **24.19.0** и `npm ci` из существующего `package-lock.json`.
   Tauri CLI устанавливается как уже закреплённая npm devDependency.
3. Явная проверка TypeScript через локальный `vue-tsc`.
4. Проверка MSVC x64/x86 и Windows SDK на Microsoft-hosted `windows-2022`.
   Rustup, Visual Studio Build Tools и SDK уже входят в этот образ; отсутствие
   компонентов вызывает понятную ошибку вместо скачивания стороннего setup-скрипта.
5. Установка официальным rustup Rust **1.99.1** и target
   **x86_64-pc-windows-msvc**. Номера Node/Rust фиксированы, но ещё не являются
   подтверждённой Windows-матрицей совместимости этого проекта.
6. `npm run tauri -- build --ci --target x86_64-pc-windows-msvc --bundles "nsis,msi" -- --locked`.
   Существующий `beforeBuildCommand` запускает `npm run build`: это ещё одна
   проверка TypeScript и production Vite build перед компиляцией Rust.
   `--locked` передаётся Cargo после `--`, не обновляя `Cargo.lock`.
7. Bundler Tauri подготавливает WiX/NSIS из предусмотренных им источников.
   Это инструменты упаковки, а не игровые ресурсы из `assets.zip` автора.
8. Проверка неизменности обоих lock-файлов; сбор обоих установщиков, EXE,
   лицензии, метаданных и SHA-256; загрузка GitHub Actions Artifact на **14 дней**.

Полные SHA закрепляют используемые официальные actions. Версии инструментария,
runner image и хеши lock-файлов записываются в `BUILD_INFO.json`. Образ
`windows-2022`, системные компоненты и источники упаковочных инструментов всё ещё
обновляются: побитовая воспроизводимость этим workflow не обещается.

## Запуск и первое добавление workflow

У GitHub есть существенное ограничение: для `workflow_dispatch` файл workflow
должен присутствовать в default branch репозитория. Добавления нового файла
только в feature-ветку недостаточно для появления рабочего `Run workflow`.

Первую проверку без merge можно получить через обычный **Pull Request**:

1. Отправить ветку `ci/windows-test-build` в `deanwell15/sattatracker-TLI`.
2. Создать PR с base repository **deanwell15/sattatracker-TLI**, base **main**.
   Не выбирать репозиторий первоначального автора как base fork network.
3. Событие `pull_request` запускает Windows job; при необходимости владелец
   разрешает GitHub Actions/первый run в настройках своего форка.
4. Проверить run и артефакты. Создание PR и merge — отдельные действия;
   эта инструкция не подразумевает автоматического merge.

После согласованного появления workflow в default branch ручной запуск:

1. Открыть **Actions → Windows test build → Run workflow**.
2. Выбрать нужную ветку в `Use workflow from` и нажать `Run workflow`.
3. Либо выполнить с установленным и авторизованным GitHub CLI:

   ```shell
   gh workflow run windows-test-build.yml --repo deanwell15/sattatracker-TLI --ref ci/windows-test-build
   ```

Если workflow ещё отсутствует в default branch, ошибка API/отсутствие кнопки
не означает ошибку компиляции приложения. Не нужно ради запуска обходить это
ограничение через `pull_request_target` или самовольно менять main.

Для PR checkout обычно соответствует синтетическому merge commit. Точный
скомпилированный commit записывается в артефакте, а не выводится из имени ветки.

## Получение и проверка приложения

В завершённом run открыть **Summary → Artifacts** и скачать:

```text
sattatracker-windows-x64-<run_id>-<run_attempt>.zip
├── installers/
│   ├── msi/*.msi
│   └── nsis/*.exe
├── standalone/tli-sattatracker.exe
├── BUILD_INFO.json
├── SHA256SUMS.txt
├── RUNNING.txt
└── LICENSE
```

Если хотя бы один ожидаемый установщик или приложение не получен, сбор артефактов
завершится ошибкой. Артефакт публикуется только после успешных предыдущих шагов.
Отсутствие файла при красном run не следует выдавать за успешную сборку.

Скачать через CLI также можно после получения ID реального run:

```shell
gh run download <run_id> --repo deanwell15/sattatracker-TLI
```

Проверка отдельного файла в PowerShell:

```powershell
Get-FileHash .\standalone\tli-sattatracker.exe -Algorithm SHA256
```

Сверить результат с соответствующей строкой `SHA256SUMS.txt`. Манифест охватывает
все остальные файлы артефакта, включая метаданные; сам себя он не хеширует.
Хеш подтверждает согласованность файлов, но не заменяет Authenticode-подпись.

## Установщики и portable EXE

MSI и NSIS — два варианта установки одного приложения. Для первой проверки
выбрать один из них и использовать чистую Windows VM/отдельное тестовое окружение.

`standalone/tli-sattatracker.exe` — **экспериментальный запуск без установки**, а
не полностью portable/offline-дистрибутив:

- Нужен установленный Microsoft Edge WebView2 Runtime. Установщики могут получить
  его через обычный механизм Tauri; отдельно скопированный EXE этого не делает.
- Release-манифест по-прежнему запрашивает администратора.
- `config.json` сохраняется возле EXE, другие данные — в LocalAppData/WebView.
- Цена, графические ресурсы и шрифты всё ещё используют прежние сетевые механизмы.
- Весь `assets.zip` в артефакт не включается и в репозиторий не переносится.
- Идентификатор приложения остаётся прежним: тестовая и существующая установка
  могут разделять данные. Не запускать их одновременно.

Для настоящего portable-режима потребуется отдельное согласованное проектирование
хранения данных и WebView2, затем динамическая проверка. Этот PR этого не делает.

## Права и публикация

Новый workflow ограничен `contents: read`, выполняется только в указанном форке,
не использует release/signing secrets, не вызывает GitHub Releases API, не пушит
изменения и не запускает собранное приложение. Используется `pull_request`, а
не `pull_request_target`. Для fork PR действуют стандартные approval-правила GitHub.

Существующий [release.yml](../.github/workflows/release.yml) **не изменён** и
по-прежнему реагирует на теги `v*`. Не создавать/не отправлять такие теги для этой
тестовой сборки: запускать именно `Windows test build`.

Статистика, parser, productName, identifier, пути данных, manifest, иконки,
текущие runtime endpoints и MIT-уведомления сохранены. План следующих изменений:
[independence-plan.md](independence-plan.md).
