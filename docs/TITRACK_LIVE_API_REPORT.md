# TITrack live API — история диагностики подключения

> Исторический отчёт для коммита ca2a7c7, до подтверждения пользователем HTTP 200.
> Текущая реализация Automatic включена: [новый отчёт](TITRACK_INTEGRATION_REPORT.md).

Дата проверки: **20 октября 2026, 12:47:24 (Asia/Yerevan)**.
PR: [#4](https://github.com/deanwell15/sattatracker-TLI/pull/4).
Ветка: `feature/deanfinite-pricing`.

**Подключение остановлено до подтверждения живого API.** Прокси Codex отказал
при создании HTTPS CONNECT-туннеля: `403 Forbidden`. Это не ответ Supabase и
не доказательство отказа анонимному SELECT. Runtime приложения в этом продолжении
не изменён. Пользователь отдельно подтвердил Windows-сборку предыдущего коммита.

## 1. Реальный статус запроса

Параметры первой проверки: HTTPS, GET `/rest/v1/aggregated_prices`,
`season_id=eq.1601`, ограниченный select восьми рыночных полей, `limit=1`,
`order=config_base_id.asc`, timeout 15 секунд; publishable key передавался только
в `apikey` header. Ключ прочитан непосредственно из актуального исходника TITrack,
не импортирован/исполнен как программа и не выведен в логи.

```json
{
  "server_http_status": null,
  "transport_error": "<urlopen error Tunnel connection failed: 403 Forbidden>"
}
```

В управляемом окружении `network_policy.state=enforced`, режим `restricted`;
`qhjulyngunwiculnharg.supabase.co` отсутствует в effective_allowed_hosts.
Ответ API, его JSON и rate-limit headers не получены. Ограничение прокси не
обходилось, сетевые настройки не менялись, повторных запросов к Supabase не было.

## 2. Подтверждение доступа к aggregated_prices

**Не подтверждён живым запросом.** В исходном SQL TITrack есть public SELECT
policy для anon/authenticated. Это свидетельство предусмотренного доступа,
но результат из текущего окружения не позволяет проверить его фактическую работу.
Нельзя интерпретировать CONNECT 403 как Supabase HTTP 403 или запрет оператора.

Актуальный HEAD TITrack проверен: `7798e02db8cfc5fb3b5eb927e3600fef99815b5e`.
`DEFAULT_SUPABASE_URL` совпадает с ожидаемым HTTPS host, ключ имеет тип
`sb_publishable_...`. service_role, JWT пользователей и закрытые credentials не
использовались. Публичный ключ в отчёт/репозиторий Deanfinite не скопирован.

## 3. Наличие живых цен SS14

**Не проверено:** ни строк, ни пустого массива от Supabase не получено.
Fixtures предыдущего этапа не выдаются за реальные рыночные данные.
Критерий «Automatic реально получает цены SS14» остаётся невыполненным.

## 4. Число доступных ID

**Неизвестно.** Нулевая доступность не утверждается. Подготовленная Windows
диагностика просит `Prefer: count=exact` в том же однострочном GET и при наличии
`Content-Range` сохраняет total_ids_if_reported. Если сервер не вернёт total,
скрипт оставит null, а не выдумает количество. Отдельных запросов bulk/RPC нет.

## 5. Поля и season mapping

Исследованная структура aggregated_prices:
config_base_id, season_id, price_fe_median, price_fe_p10, price_fe_p90,
submission_count, unique_devices, updated_at. Фактически возвращаемые поля
пока не проверены. Диагностика выведет returned_fields и один sample только
с этими рыночными колонками.

В player_parser.py TITrack SS14 Tartarus обозначен 1601. Явный mapping в
Deanfinite уже задан как 1601 → 1601; это проверка исходников, не подтверждение
наличия рыночных строк на сервере.

Исходники TITrack, использованные в проверке:

- [client.py](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/src/titrack/sync/client.py#L66): публичная конфигурация; L189 чтение агрегатов.
- [manager.py](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/src/titrack/sync/manager.py#L60): 300 секунд.
- [SQL schema](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/supabase/migrations/001_initial_schema.sql#L45): поля; L268 SELECT policy.
- [season mapping](https://github.com/astockman99/TITrack/blob/7798e02db8cfc5fb3b5eb927e3600fef99815b5e/src/titrack/parser/player_parser.py#L51).

## 6. HTTPS-клиент диагностики

[tools/check-titrack-api.py](../tools/check-titrack-api.py) — автономная утилита
Python **3.10+**, только стандартная библиотека, совместимая с Windows.
Она делает два GET к GitHub: текущий commit master и pinned client.py; Python-код
TITrack не выполняется, literal constants читаются через ast. Затем выполняется
**один GET к фиксированному Supabase host**, без retry. Key добавляется только
в этот запрос, не в GitHub headers. TLS verification и системный proxy сохранены.

Redirect запрещён; произвольных URL/ключей через параметры нет. GitHub responses
ограничены 256 KiB, API response — 64 KiB; каждый запрос ограничен 15 секундами.
Ни один результат не импортируется автоматически в кэш приложения.

200 и 206 с массивом считаются успешным SELECT; пустой массив — доступный API,
но отсутствие строк SS14. 401/403/429/500/503 показываются как HTTP error без
обхода/повторов; Retry-After и известные rate-limit headers сохраняются. Ошибка
CONNECT/DNS/timeout оставляет server_http_status=null с явным phase. Ключ
редактируется даже в неожиданном тексте ошибки. Ответы не содержат логи игры,
инвентарь, drop history или пользовательские ID.

## 7. Pagination и delta sync

В production **не реализованы в этом продолжении**: первая живая проверка не
прошла границу окружения. Предыдущая архитектура и fixture adapter сохранены.
Диагностика намеренно ограничена одной строкой и не выдаётся за полную синхронизацию.
После успешной проверки потребуется транспорт: страницы до 1000, стабильный порядок,
ограничение страниц/байтов/ID, атомарное применение всех страниц, cursor из max
updated_at успешно прочитанных строк с защитой равных timestamp и полная сверка.

## 8. Кэш

Существующий season-scoped кэш PR #4 не менялся. При ошибке сети сохраняет
предыдущие данные; Local JSON продолжает работать offline. Старый сервер или
environment override не возвращены. Ни один live sample в кэш не записан.

## 9. Цикл 5 минут

Существующий интервал **300 секунд** сохранён. Automatic по-прежнему сообщает
production gate; успешная сеть не заявляется. Timeout/retry/backoff транспорта
нужно реализовать только после успешного live probe; ручная диагностика выполняет
одну попытку и не повторяет запрос при 429, поэтому не нарушает Retry-After.

## 10. Что отправляется

GitHub: URL публичного исходника и commit metadata, стандартные GET headers.
Supabase: только параметры таблицы/сезона/select/limit/order, `apikey`, Accept,
User-Agent (и в Windows-пробе Prefer для count). В текущей попытке CONNECT был
отклонён до получения ответа Supabase. POST/PATCH/DELETE/RPC и submit_price не
вызывались. Идентификаторов устройства/пользователя, логов и наблюдений нет.

## 11. Ограничения и точный следующий шаг

Блокирующий фактор — **сетевой доступ текущего Codex окружения к ожидаемому host**.
Это отличается от подтверждённого запрета TITrack. Доступность API, SS14 данные,
квота и дополнительные условия пока неизвестны.

Для продолжения нужен результат Windows-пробы либо разрешение ожидаемого host
в настройках сети Codex через штатную конфигурацию окружения. До этого runtime
не включается, согласно явному условию задачи «подготовь диагностику и остановись».

## 12. Тесты

У новой утилиты пройдены **8 контролируемых unit tests** (без сети):
200/206 и поля/count; пустой массив; CONNECT 403 и timeout; 401/403/429/500/503;
malformed/oversized JSON; неверный сезон/форма ответа; запрет redirect/secret key/чужого host. Также проверены
только GET, один Supabase request, отсутствие key в GitHub headers/выводе.
Это тесты диагностической утилиты, не завершение списка тестов HTTP-провайдера.

Бизнес-логику, парсер, графику и исходники приложения не меняли; повторный запуск
их локальных тестов не нужен. Ранее для 12a53f2 успешно прошли 22 native Rust tests,
9 Pinia scenarios, 6 Chromium scenarios и Windows packaging; это прежний результат,
не доказательство работающего live TITrack.

## 13. Windows CI

Существующий workflow сохранён; добавлен только запуск
`python tools/check-titrack-api.py --self-test` с контролируемыми ответами.
CI **не вызывает настоящий Supabase** и не зависит от его доступности.
Обновление PR автоматически запускает Windows CI; текущий результат следует
проверять по [Actions ветки](https://github.com/deanwell15/sattatracker-TLI/actions/workflows/windows-test-build.yml?query=branch%3Afeature%2Fdeanfinite-pricing).

Предыдущий проверенный native run:
[39386021591 — success](https://github.com/deanwell15/sattatracker-TLI/actions/runs/39386021591).
Успех новой проверки не утверждается до её фактического завершения.

## 14. Как проверить на Windows

Нужен установленный Python 3.10+; сторонние Python-пакеты не требуются.
Получить tools/check-titrack-api.py из текущей ветки PR #4, открыть PowerShell
в папке checkout и выполнить:

```powershell
py -3 --version
py -3 tools/check-titrack-api.py --self-test
$diagnosticFile = Join-Path ([Environment]::GetFolderPath('Desktop')) 'TITRACK_API_DIAGNOSTICS.json'
py -3 tools/check-titrack-api.py --output $diagnosticFile
```

Если Python доступен как `python`, использовать его вместо `py -3`.
Результат сохраняется на **реальный рабочий стол**, включая перенаправление OneDrive.
Ручной запуск делает один Supabase GET и два GitHub GET; не запускать в цикле.
Программа не устанавливает зависимости и не запускает сторонний код.

Проверить: phase=supabase, server_http_status=200/206, anonymous_select_confirmed=true,
ss14_rows_found, total_ids_if_reported, returned_fields, sample, headers.
200 с пустым массивом означает API доступен, но SS14 строк не найдено.
401/403 — остановиться; 429 — соблюдать Retry-After; transport error — проверить
обычный доступ к сети, не отключать TLS и не обходить политику сервера.
Результат не содержит key; его можно предоставить для следующего согласованного шага.

До дальнейших инструкций остановка. Merge и GitHub Release не выполнялись.

## Изменённые файлы

- `tools/check-titrack-api.py` — ручная диагностика и автономные тесты.
- `docs/TITRACK_LIVE_API_REPORT.md` — этот отчёт.
- `.github/workflows/windows-test-build.yml` — только offline self-test утилиты.
