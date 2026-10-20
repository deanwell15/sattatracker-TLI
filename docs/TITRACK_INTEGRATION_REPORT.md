# Deanfinite — реальные агрегированные цены TITrack

Дата: 20 октября 2026 (Asia/Yerevan). Ветка `feature/deanfinite-pricing`,
[PR #4](https://github.com/deanwell15/sattatracker-TLI/pull/4). Merge/Release запрещены и не выполнялись.

## 1. Живой HTTP status: подтверждено пользователем

**Пользователь выполнил проверку на своём Windows: HTTP 200 OK**, публичный
publishable key из TITrack работает, aggregated_prices доступна на чтение,
для SS14/1601 есть рыночные записи. Переданная запись:

| Поле | Значение |
|---|---|
| config_base_id | 350714 |
| season_id | 1601 |
| price_fe_median | 388 FE |
| updated_at | 2026-10-20T08:45:00.442384+00:00 |

Это данные сообщения пользователя, **не результат запроса Codex/CI**.
Ранее Codex получил `Tunnel connection failed: 403 Forbidden` от собственного
прокси. Ответа Supabase в той попытке не было; этот факт не опровергает Windows
HTTP 200. Повторных попыток обхода облачной сетевой политики не делалось.

## 2. Доступ и реальные цены SS14

Доступность таблицы и хотя бы одной записи SS14 подтверждены пользователем.
Production gate удалён, Automatic действительно вызывает HTTPS transport.
**Проверка нового Deanfinite на живом API после установки остаётся ручной:**
CI не получает реальные рыночные данные и не доказывает end-to-end работу в игре.

## 3. Количество ID сезона

Полное число ID по сообщению пользователя неизвестно; одна запись не даёт
численность таблицы. Фиктивное число не подставлялось. После успешной полной
загрузки приложение показывает Cloud cache IDs отдельно от локального fallback
и общего числа оценённых ID (с FE). В cache count могут оставаться прежние записи:
пропуск ID в delta/full не стирает известную оценку, которая со временем Stale.
Read-only diagnostic может получить total из Content-Range при count=exact,
но сама эта дополнительная проверка из Codex не выполнялась.

## 4. Формат и источник

Endpoint: `https://qhjulyngunwiculnharg.supabase.co/rest/v1/aggregated_prices`.
GET select: config_base_id, season_id, price_fe_median, price_fe_p10,
price_fe_p90, submission_count, unique_devices, updated_at. Основная цена — median.
Пользователь явно подтвердил четыре поля из таблицы выше; остальные входят в
проверенный контракт исходников TITrack и покрыты controlled tests.

TITrack агрегирует аукционные наблюдения своего сообщества. Deanfinite не собирает
такие наблюдения и не вызывает submit_price. Это оценки предложений, не цены
подтверждённых сделок. updated_at — время агрегации, не время последней исходной
сделки/наблюдения. Дополнительный клиентский порог >=3 devices удалён: счётчики — необязательные
метаданные сервера. Структурно невалидный batch по-прежнему отклоняется целиком.
См. [исправление синхронизации и Fast Build](TITRACK_SYNC_FAST_BUILD_REPORT.md).

Публичная конфигурация взята из
[client.py @ 7798e02](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/src/titrack/sync/client.py#L66).
Published key включён только как публичная конфигурация backend; это не secret или
service_role. Значение не выводится в отчёт, HTTP ошибки или диагностические логи.
Для ротации ключа нужен проверенный update исходников и новая сборка; произвольного
runtime URL/env override нет. Файл конфигурации отделён от кода transport.

Явный season mapping: Deanfinite 1601 → TITrack 1601, согласно TITrack
[player_parser.py](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/src/titrack/parser/player_parser.py#L51).
Другие сезоны не запрашиваются без подтверждённого mapping.

## 5. HTTPS-клиент

[src-tauri/src/pricing/titrack.rs](../src-tauri/src/pricing/titrack.rs) реализует
существующий PriceProvider. reqwest 0.13.2 с blocking+rustls вызывается **только
через Tokio spawn_blocking**. UI/event loop не ожидает HTTP; operation mutex и
valuation RwLock освобождены до обращения к provider. Отдельный atomic flag
исключает параллельные sync, generation отменяет результаты после смены сезона
или источника. Между страницами generation проверяется без блокировок.

Production использует только фиксированный HTTPS host/path, TLS verification,
запрет redirects, `apikey`, Accept и User-Agent. Нет произвольных адресов из логов,
скачивания/запуска программ, POST/PATCH/DELETE/RPC, новых Tauri permissions.
Connect timeout 5 секунд, запрос до 10 секунд, весь sync до 45 секунд.
Максимум 1 MiB на response, 4 MiB на batch, 50 data pages + watermark query,
10 000 records за sync. Превышение/ошибка сохраняет прошлый cache.

Идентификаторы — положительные числа до 12 цифр; median/percentiles — конечные
0..1e9, сезон каждой записи должен совпасть, timestamp валидный RFC3339 с timezone.
Сохранена проверка будущей даты (не дальше +5 минут). Unknown — value:null;
Known 0 отличается от Unknown. FE всегда 1 по существующей базовой quote.

## 6. Pagination

Первая короткая GET query читает максимальный **серверный updated_at**,
`order=updated_at.desc,config_base_id.asc&limit=1`. Это upper watermark.
Далее страницы по 1000, `order=config_base_id.asc`, `config_base_id=gt.<last_id>`,
`updated_at=lte.<upper>`. Используется keyset pagination, не плавающий offset. Завершение — по пустой
странице: короткая непустая страница не считается концом, если server cap ниже 1000.

ID должны строго возрастать; дубликаты, нарушение границ, неверные rows,
ошибка промежуточной страницы или превышение лимита отклоняют весь batch.
Изменения после upper не попадают в текущий срез и читаются следующим delta.
Равные timestamps включаются повторно и безопасно объединяются по ID.

## 7. Delta и полный reconcile

После первой успешной полной загрузки cursor сохраняется **из максимального
updated_at действительно прочитанных/применённых rows**, не локального времени
запроса и не предварительного head. Точность RFC3339 до микросекунд сохранена,
сравнение выполняется по DateTime, включая разные timezone offsets.

Следующий delta использует `updated_at=gte.<cursor>` (включающую границу), ту же
верхнюю границу и keyset pages. Это предотвращает потерю записей с тем же timestamp.
Пустой delta не удаляет цены и не сбрасывает cursor. Не более старые данные
обновляют записи; более старые не заменяют новые, включая различия внутри секунды.
Полная сверка — каждые **6 часов**, а также при отсутствии пригодного cursor/full
metadata. При серверном watermark ниже cursor выполняется полный проход;
прежние более новые записи сохраняются. Серверные удаления не трактуются как
подтверждённая цена 0: сохранённая оценка продолжает стареть.

## 8. Кэш и приоритет

Cache v1 дополнен optional sync metadata: cursor, last_full_at, last_success_at.
Цены и metadata записываются **в одном атомарно заменяемом файле**
`prices/v1/<season>/automatic.json`. Ошибка HTTP/JSON/валидации/записи не меняет
цены/cursor на диске. RAM публикуется после успешного сохранения. Старый v1 cache
без metadata читается, но запускает полный sync. Cursor из повреждённого cache
не может указывать за max timestamp сохранённых записей.

Automatic: **cloud table primary**, в том числе её явно Stale оценка; Local JSON
заполняет только отсутствующие cloud ID. Fresh cloud не заменяется старым local.
Это уточняет прежний приоритет, при котором любой local мог вытеснить stale cloud.
Local JSON вообще не обращается к provider и не использует cloud estimates.
Смена режима применяет имеющиеся таблицы без повторного учёта предметов.
Сезонные caches сохраняются раздельно; старый endpoint не возвращён.

## 9. Цикл 300 секунд, offline, ошибки

UI показывается сразу; после local cache initialize публикуется snapshot **до**
первого запроса. Игровой трекер работает независимо. Source setter сохраняет выбор
и немедленно возвращает текущий snapshot, затем Notify будит фоновую задачу;
HTTP не удерживает настройки или price locks.

В норме новый запрос назначается через 300 секунд от начала предыдущего; время
сетевой работы вычитается из ожидания. Пропущенные тики не догоняются пачкой.
Нет немедленных retries. При временных ошибках задержки 300, 600, 1200, 2400,
3600 секунд, затем максимум час; Retry-After (seconds или HTTP date), отсчитанный от получения ошибки, увеличивает
задержку при необходимости. Большой Retry-After не сокращается до часа;
таймер может проверять срок чаще, но HTTP до разрешённого времени не выполняется.
401/403 приостанавливают автоматические запросы на время процесса; переключение
Local/Automatic не обходит запрет. После перезапуска возможна одна обычная попытка.
Retry cooldown хранится в RAM, не переживает перезапуск.

При ошибке cache остаётся, UI показывает offline_cache/access_denied, причину,
последнее успешное обновление и next request. Network error скрывается в Local
mode. Логи содержат HTTP status, сезон и число cached IDs; header/key не печатаются.
Подробный reqwest Request/Response Debug и error bodies не логируются.

## 10. Отправляемые данные

Только GET к таблице: season ID, select, order, page size, границы updated_at/ID,
публичный apikey, Accept, User-Agent. Ни username, ни device ID, ни игровые логи,
инвентарь, drop history, XchgSearchPrice или другие личные данные не отправляются.
GitHub/raw чтение требуется только разработчику/ручной диагностике для проверки
конфигурации; runtime приложения не скачивает код/ключ из GitHub.

## 11. Ошибки и ограничения

- Прокси Codex по-прежнему не позволяет подтвердить прямой Supabase GET; это
  ограничение окружения, а не доказательство проблемы сервера.
- Данные SS14 и HTTP 200 подтверждены пользователем, но полный count/headers/квота
  не предоставлены; отсутствие quota headers не означает отсутствие ограничений.
- 300 секунд соответствует TITrack client; явный server Retry-After имеет приоритет.
- Строгая валидация может отклонить весь batch при дефектной записи — cache
  сохраняется, ошибка видна. Не утверждается полное покрытие всех предметов.
- Stale определяется 24 часами от source updated_at; дата агрегации не раскрывает
  точный возраст исходных наблюдений.
- Старые независимые дефекты resetSession/mapCost, inventoryValue и классификации
  расходов не изменялись. Графика, парсер/FSM, брендинг и layout не менялись.

## 12. Автоматические тесты

Локально пройден **33 Rust unit/integration scenario** реальных модулей,
в том числе 18 сетевых/синхронизационных сценариев: 200 с реальными field names,
empty, 401/403, 429 numeric/date Retry-After, timeout, 500/503, redirect,
malformed/oversized JSON, invalid ID/price/season, pagination, mid-page failure,
inclusive delta, periodic full, offline cache/cursor, Local no-request,
backoff, no-lock/source-change cancellation, key redaction.

Часть тестов использует настоящий reqwest → **loopback HTTP mock server**;
остальные — контролируемый transport, не реальные рыночные данные.
Production разрешает только HTTPS; HTTP test constructor доступен лишь cfg(test)
и принимает только 127.0.0.1. Сетевые unit tests не обращаются к Supabase.
Сохраняются тесты неизвестного лута/переоценки: 9 Pinia scenarios, 6 Chromium
scenarios (Tauri IPC mocked), 3 tests EXE, TypeScript и production frontend.
Отдельная диагностическая утилита — 8 offline self-tests.

## 13. Windows CI

Workflow Windows 2022 сохранён: locks, typecheck, pricing, diagnostic self-test,
browser smoke, frontend + native Tauri build, MSI/NSIS, полный cargo test --lib,
проверка original assets внутри обоих installers и standalone kit.
**Live Supabase в CI не вызывается.** Результат Windows сборки фиксируется только
по фактически завершённому run в [PR #4](https://github.com/deanwell15/sattatracker-TLI/pull/4).
Текущий run/result будет указан в описании PR и итоговом отчёте после завершения.
Успех предыдущей сборки не выдаётся за подтверждение новой сетевой реализации.

## 14. Проверить вручную на Windows

1. Установить MSI/NSIS либо распаковать standalone **EXE + icons/** с WebView2.
2. Выбрать Settings → Filter → Market prices → **Automatic (TITrack)**.
   Существующий выбор Local JSON сохраняется; принудительно источник не меняется.
3. Убедиться: season 1601, online, last successful update, Cloud cache IDs >0;
   проверить ID 350714 и median (388 FE — подтверждённая пользователем цена
   прошлого snapshot, текущая может измениться).
4. Подобрать предмет до/после загрузки цены: qty сохраняется, сумма обновляется,
   эффекты не повторяются. Изменить Min Drop Value: только видимость feed.
5. Без сети перезапустить: cache доступен, ошибка видна, UI/лог не блокируются.
6. В Local JSON проверить нулевые HTTP requests, импорт и локальные fallback ID.
7. Убедиться в пятиминутном обновлении и сохранении количества/карты/сессии.
8. При 429 ждать Retry-After; при 401/403 не обходить запрет, проверить актуальность
   публичной конфигурации обычным read-only diagnostic и сообщить статус.
9. Проверить оригинальные изображения main/pop-out и расположение виджетов.

[Предыдущий отчёт о proxy failure](TITRACK_LIVE_API_REPORT.md) сохранён как история
диагностики до пользовательского подтверждения. Полная модель учёта — [pricing.md](pricing.md).
