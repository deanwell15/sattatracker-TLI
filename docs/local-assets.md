# Deanfinite: оригинальная графика без загрузки при запуске

PR #3, ветка `feature/deanfinite-local-assets`. Графическая задача не меняет
PricingManager, parser, FSM, BagManager, расчёты, налог, настройки, Deanfinite
или identifier `io.github.deanwell15.deanfinite`.

## Источник и состав

Архив https://github.com/sattatrapy/sattatrackerUpdate/releases/download/Assets/assets.zip
получен 19.10.2026: 189 021 488 байт. SHA-256 совпал с digest GitHub Release:
`83e55bb0e7d193e43a66fa2304f5eff867360fc10b51ee29b45d74ac708669ab`.
Проверены CRC всех записей, пути, отсутствие symlink/дубликатов/шифрования,
ограничения размера и полное декодирование каждого изображения Pillow.
Никакие файлы из архива не выполнялись.

| Набор | Количество | Байты | Упаковка |
|---|---:|---:|---|
| Иконки | 2 845 (2 844 числовых ID + fe) | 60 558 114 | Tauri bundle resource `icons/` |
| Свинка | 121, из них используются 120 | 91 909 865 | Vite frontend, встроен в EXE |
| Монеты | 122 | 37 528 797 | Vite frontend, встроен в EXE |
| Всего | 3 088 | 189 996 776 | Смешанная схема |

Все файлы сохранены без перекодирования. Расширение .png не всегда отражает
содержимое: 1 652 файла WebP, 1 436 PNG. PNG-анимации 1664×1244; индивидуальные
параметры находятся в [manifest](../assets/original-assets.manifest.json).
В архиве есть запасной pig_120.png: он сохранён, но цикл по-прежнему использует
ровно pig_0…119. Для каждого из четырёх tier все 30 кадров различаются по
декодированным пикселям; все 122 coin frames также различаются.

Иконки используются LootFeed, ItemsHistory и ExtraEffectsOverlay. Анимации —
PiggyWidget и coinParticles ExtraEffectsOverlay. Исходные FPS=30 в PiggyWidget,
комбо, уровни, bounce, idle fade, timeout, клик и физика дополнительных эффектов
сохранены без изменения их скриптов.

Лицензии/атрибуция: [ASSET_LICENSES.md](../ASSET_LICENSES.md). LICENSE/NOTICE в
архиве нет; конкретного запрета в проверенных материалах не найдено. Права на
чужой art не присваиваются и MIT исходников на него автоматически не расширяется.

## Размер Git

Около 181,2 MiB исходных изображений добавляется обычными Git blobs. Крупнейшие
отдельные файлы меньше 4 MiB, ограничение GitHub 100 MiB/file не затрагивается.
LFS, перекодирование и оптимизация хранения не внедрялись. Первый clone/checkout
и CI artifact заметно вырастут; дальнейшие изменения кадров добавят новые blobs
в историю. При необходимости LFS/оптимизацию согласовать отдельно. Сам ZIP в Git
не добавлен: хранится распакованный проверяемый набор.

## getIcon / getAnim и безопасность

`getIcon(id)` синхронно проверяет ID в `src/assets/original-icons.json` и
возвращает URL собственного `deanfinite-icon` protocol с ревизией источника.
Tauri на Windows формирует `http://deanfinite-icon.localhost/...` — это локальный
WebView protocol, не сетевой сервер и не внешний HTTP-запрос. Зарегистрирован
для всех WebView, поэтому отдельные окна имеют тот же доступ.

Обработчик разрешает только `/[0-9]{1,12}.png` и `/fe.png` внутри
`resource_dir()/icons`. Traversal, encoded separators, дисковые пути, symlinks,
каталоги и oversized файлы отклоняются. Ответ использует MIME по фактическим
байтам и nosniff. Файловый assetProtocol по-прежнему выключен, scope пуст;
произвольные пользовательские каталоги не открываются.

`getAnim(path)` сопоставляет старые pig_N / coin/coin_N с eager Vite imports,
исходные PNG публикуются Vite с хешированными именами и встраиваются Tauri в EXE.
В компонентах не меняются порядок кадров или реакция на игровые события.

Неизвестный ID/кадр → item.svg; ошибка чтения/декодирования → AssetImage с
локальным fallback, затем CSS ◇, если и fallback повреждён. Это аварийный путь.
Для имеющихся ресурсов отображаются оригинальные предметные изображения и кадры.
Никаких metadata/ZIP/updater/авторских Release-запросов в runtime нет. UI не ждёт
графику или LoadingDone; состояние чтения лога остаётся неблокирующим.

## Покрытие и SS14

В нашем существующем en_id_table.json 2 594 ID; архив покрывает 2 527, отсутствует
67. Это не основание менять names/классификацию в графическом PR.
Сравнение с уже исследованным TITrack seed commit 7798e02 (SS14 update включает
также SS13 backfill) показывает 25 отсутствующих ID из 256 добавленных:

```text
10330 10331 10332 112358 112444 113287 113288
396010 396011 396012 396013
396500 396501 396502 396510 396511 396512
396520 396521 396522 396530 396531 396532
502000 502001
```

Полные ID-списки: [original-assets-coverage.json](original-assets-coverage.json).
Это сравнение наличия файлов, не доказательство правильности всех сезонных
соответствий: игра переиспользует ID. Полноценный ItemCatalog и изменение
классификации сейчас не внедряются. Часть SS14 изображений остаётся отсутствующей;
их нельзя выдавать за восстановленные или тайно заменять выдуманной картинкой.

`tools/plan-tlidb-icon-update.py` принимает локальный справочный JSON и создаёт
план недостающих CDN mappings без сетевых запросов/копирования чужого seed в
runtime. Для следующего обновления нужны подтверждённые условия TLIDB и отдельная
проверка season=1601/mapping/прав. Пока автоматическое получение выключено.

## Сборка, установщики и standalone

Tauri map `resources/icons/*.png → icons/` включает все иконки в MSI/NSIS и
копирует их рядом с release EXE. Анимации встроены во frontend/EXE. Standalone
artifact **обязательно** содержит комплект:

```text
standalone/
  Deanfinite.exe
  icons/
    100300.png
    ...
```

Не копировать один EXE без icons. Полностью автономный единый EXE не заявляется.
WebView2, существующие elevated права и AppConfig/LocalAppData также требуются;
цены теперь используют локальный JSON/кэш, TITrack gated до согласования доступа. Для offline-графики не нужен download, но установка
самого WebView2 и свежие цены — отдельные вопросы.

CI проверяет hashes всех resource-копий, распаковывает MSI CAB и NSIS через
предустановленный 7-Zip, сверяет все 2 845 иконок с исходным release. Для EXE допускается только точная
служебная замена первого bundle-type marker Tauri UNK → MSI/NSS; все остальные
байты, включая анимации, должны совпадать.
Сам установщик при проверке содержимого не запускается. Standalone kit также
проверяется по hashes. ORIGINAL_ASSETS.json, ASSET_LICENSES.md и SHA256SUMS
добавлены в artifact. Автоматических GitHub Release нет.

## Автоматические проверки

- `npm run test:assets`: исходные hashes/реальные MIME всех 3 088 файлов;
  registry, наличие кадров и различие decoded hashes для 4×30 + 122;
  6 Chromium-сценариев: clean, restart, offline boundary, missing, corrupt,
  widgetOnly. Реальный Vue/Pinia с Tauri mocks; проверяются оригинальные URL,
  разные иконки, полный порядок frame mapping, продвижение кадров PiggyWidget
  и coin sequence после клика; корректные текст/количество/сумма остаются.
- `npm run build`: vue-tsc/Vite и `check-frontend-assets.mjs` — побайтное наличие
  всех 243 оригинальных файлов анимаций в production frontend.
- Rust `icon_resource::tests`: допустимые MIME/байты; traversal/encoded/drive/
  oversized/invalid file; Unix symlink fixtures, когда платформа их поддерживает.
- Windows CI: settings и icon_resource tests, MSI/NSIS и standalone contents.
- Исходный архив при импорте: SHA-256/CRC, строгие пути и limits, полное
  декодирование, pixel hashes. Developer-only importer воспроизводим:

```shell
python tools/import-original-assets.py --archive path/to/assets.zip
# Для разработчика, если ZIP ещё не скачан (не приложение и не CI):
python tools/import-original-assets.py --download --archive path/to/assets.zip
```

Импорт требует Pillow; запуск приложения/CI build его не требует. Никакой
скачанный executable не выполняется. Старые пользовательские public_assets/ZIP/
metadata не читаются и не удаляются.

## Ручная проверка Windows

1. Дождаться зелёного run актуального commit в Actions → Deanfinite Windows test
   build. Проверить BUILD_INFO/ORIGINAL_ASSETS/SHA256SUMS.
2. В тестовой VM/новом профиле с WebView2 отключить сеть до первого запуска.
   Установить один MSI/NSIS либо распаковать **весь** standalone kit.
3. UI, F8, основные виджеты и графика доступны без cache/download. Отсутствие
   игры/свежих цен не должно скрывать widgets; не считать неизвестные цены дефектом
   упаковки графики.
4. На игровом логе сверить разные знакомые item icons, историю, session/map loot.
   Сверить свинку всех четырёх tier, bounce/fade/combo timeout и исходную анимацию
   монет после клика, частицы/дополнительные эффекты. Триггеры не менялись.
5. Открепить/вернуть окна и повторить проверку offline. Это требует реального
   WebView2; браузерный mock не доказывает нативную работу окон.
6. Закрыть/повторно открыть приложение из другого CWD. Standalone icons остаются
   рядом с EXE. Не переносить user cache ради исправления package.
7. Сверить отсутствие запросов к авторским Release/assets.zip во всех окнах;
   прежние price requests отделять от графики. Старые пользовательские файлы
   до/после запуска должны остаться неизменны.
8. Проверить отсутствующие SS14 ID: понятный аварийный fallback, текст/числа целы.
   Не объявлять отсутствующие оригиналы восстановленными.

Причина различия EXE подтверждена исходниками Tauri CLI 2.10.1:
[bundle.rs::patch_binary](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.10.1/crates/tauri-bundler/src/bundle.rs#L22).
Проверка marker и отказ при изменении другого байта покрыты Node unit tests.
