SELECT DISTINCT f1.flight_id, f1.aircraft_code, f2.flight_id, f2.aircraft_code, f1.departure_airport, f1.arrival_airport, f2.arrival_airport
FROM flights_v f1
         JOIN flights_v f2 ON f2.departure_airport = f1.arrival_airport
WHERE f1.status = 'Scheduled' AND f2.status = 'Scheduled'
  AND f1.departure_airport = 'KEJ' AND f2.arrival_airport = 'GDZ'
  AND (f1.scheduled_departure BETWEEN '2017-08-15 04:20:00.000000 +00:00' AND '2017-08-21 04:20:00.000000 +00:00')
  AND (f2.scheduled_departure BETWEEN f1.scheduled_arrival AND f1.scheduled_arrival + INTERVAL '24h');

-- flight_id, fare_conditions

DROP FUNCTION IF EXISTS aircraft_type(integer);
DROP FUNCTION IF EXISTS occupied_seats(integer, varchar);
DROP FUNCTION IF EXISTS free_seats(integer, varchar, varchar);

-- flight_id
CREATE FUNCTION aircraft_type(integer) RETURNS varchar
AS '
    SELECT aircraft_code FROM flights_v
    WHERE flight_id = 60622
'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;

-- flight_id, fare_conditions
CREATE FUNCTION occupied_seats(integer, varchar) RETURNS integer
AS '
    SELECT count(ticket_no) AS occupied_seats
    FROM ticket_flights
    WHERE flight_id = $1
      AND fare_conditions = $2
'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;

-- occupied_seats, aircraft_code, fare_conditions
CREATE FUNCTION free_seats(integer, varchar, varchar) RETURNS integer
AS '
    SELECT count(seat_no) - $1 as free_seats
    FROM seats_comfort
    WHERE seats_comfort.aircraft_code = $2
      AND fare_conditions = $3
'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;

SELECT * FROM aircraft_type(4154);
SELECT * FROM occupied_seats(4154, 'Economy');
SELECT * FROM free_seats(50, '773', 'Economy');

SELECT * FROM free_seats(occupied_seats(4154, 'Economy'), aircraft_type(4154), 'Economy');

CREATE INDEX ticket_flights_index ON ticket_flights (flight_id, fare_conditions) WITH (deduplicate_items = off);

DROP TABLE IF EXISTS free_seats;

CREATE TABLE free_seats
(
    flight_id       INT                     NOT NULL,
    fare_conditions VARCHAR(10)             NOT NULL,
    free_seats      INT                     NOT NULL,

    PRIMARY KEY (flight_id, fare_conditions)
);

INSERT INTO free_seats(flight_id, fare_conditions, free_seats)
SELECT flight_id, 'Economy' AS fare_conditions, free_seats
FROM flights_v
         CROSS JOIN LATERAL free_seats(occupied_seats(flight_id, 'Economy'), aircraft_type(flight_id), 'Economy') AS free_seats
WHERE status = 'Scheduled';

SELECT flight_id, 'Economy' AS fare_conditions, occupied_seats
FROM flights_v
         CROSS JOIN LATERAL occupied_seats(flight_id, 'Economy') AS occupied_seats
WHERE status = 'Scheduled';

-- CREATE FUNCTION free_seats(integer, VARCHAR) RETURNS integer
-- AS '
--     WITH occupied_seats_query AS (SELECT count(fare_conditions) as occupied_seats
--                                   FROM ticket_flights
--                                            JOIN flights_v ON flights_v.flight_id = ticket_flights.flight_id
--                                   WHERE flights_v.flight_id = $1
--                                     AND fare_conditions = $2
--                                   GROUP BY fare_conditions),
--          aircraft_code_query AS (SELECT DISTINCT aircraft_code
--                                  FROM ticket_flights
--                                           JOIN flights_v ON flights_v.flight_id = ticket_flights.flight_id
--                                  WHERE flights_v.flight_id = $1)
--     SELECT (count(fare_conditions) - occupied_seats_query.occupied_seats) as free_seats
--     FROM seats_comfort,
--          occupied_seats_query,
--          aircraft_code_query
--     WHERE seats_comfort.aircraft_code = aircraft_code_query.aircraft_code
--       AND fare_conditions = $2
--     GROUP BY fare_conditions, occupied_seats_query.occupied_seats;
-- '
--     LANGUAGE SQL
--     IMMUTABLE
--     RETURNS NULL ON NULL INPUT;

WITH occupied_seats_query AS (
    SELECT count(fare_conditions) as occupied_seats
    FROM ticket_flights
             JOIN flights_v ON flights_v.flight_id = ticket_flights.flight_id
    WHERE flights_v.flight_id = 4133 AND fare_conditions = 'Comfort'
    GROUP BY fare_conditions
), aircraft_code_query AS (
    SELECT DISTINCT aircraft_code
    FROM ticket_flights
             JOIN flights_v ON flights_v.flight_id = ticket_flights.flight_id
    WHERE flights_v.flight_id = 4133
)
SELECT (count(fare_conditions) - occupied_seats_query.occupied_seats) as free_seats
FROM seats_comfort, occupied_seats_query, aircraft_code_query
WHERE seats_comfort.aircraft_code = aircraft_code_query.aircraft_code AND fare_conditions = 'Comfort'
GROUP BY fare_conditions, occupied_seats_query.occupied_seats;

WITH RECURSIVE flights_recur AS (
    SELECT
        ARRAY[f1.departure_airport]::VARCHAR[] AS path,
            f1.arrival_airport AS end_point,
        f1.scheduled_departure AS departure_time,
        f1.scheduled_arrival AS arrival_time,
        ARRAY[f1.flight_id]::INT[] as flight_id,
        -- ARRAY[prices.amount]::INT[] as prices,
            0 as len
    FROM flights_v f1
             JOIN prices ON f1.flight_no = prices.flight_no
        AND prices.fare_conditions = 'Economy'
    WHERE f1.departure_airport = ANY('{DME,SVO}'::VARCHAR[])
      AND f1.status = 'Scheduled'
      AND f1.scheduled_departure
        BETWEEN '2017-08-15 04:20:00.000000 +00:00'
        AND '2017-08-21 04:20:00.000000 +00:00'
      AND free_seats(occupied_seats(f1.flight_id, 'Economy'), aircraft_type(f1.flight_id), 'Economy') > 1

    UNION

    SELECT
        flights_recur.path || flights_recur.end_point,
        fn.arrival_airport,
        flights_recur.departure_time,
        fn.scheduled_arrival,
        flights_recur.flight_id || fn.flight_id,
        -- flights_recur.prices || prices.amount,
        len + 1 AS i
    FROM flights_recur
             JOIN flights_v fn ON (
        flights_recur.end_point = fn.departure_airport
            AND fn.status = 'Scheduled'
            AND NOT (fn.arrival_airport = ANY(flights_recur.path))
            AND fn.scheduled_departure
            BETWEEN flights_recur.arrival_time
            AND flights_recur.arrival_time + INTERVAL '24h'
            AND free_seats(occupied_seats(fn.flight_id, 'Economy'), aircraft_type(fn.flight_id), 'Economy') > 1
        )
    --     JOIN prices ON fn.flight_no = prices.flight_no
--         AND prices.fare_conditions = 'Business'
    WHERE len < 1
)
SELECT
    flights_recur.path || flights_recur.end_point AS path,
    flights_recur.flight_id as flight_ids,
    -- flights_recur.prices,
    departure_time,
    arrival_time,
    len AS connections
FROM flights_recur
WHERE flights_recur.end_point = ANY('{GDZ}'::VARCHAR[]);

SELECT NOT ('DME' = ANY('{KEJ,OVB,DME,GDZ}'::VARCHAR[]));

SELECT flights_v.aircraft_code, departure_airport, arrival_airport, scheduled_departure, scheduled_arrival
FROM flights_v
WHERE flight_id = 4133;

SELECT flights_v.flight_id, prices.amount
FROM flights_v
         JOIN prices ON flights_v.flight_no = prices.flight_no
WHERE flights_v.flight_id = ANY('{47317, 60622}'::INT[])
  AND fare_conditions = 'Economy';

SELECT fare_conditions FROM ticket_flights
WHERE ticket_no = '0005435208229' AND flight_id = 60731;

SELECT aircraft_code FROM flights_v
WHERE flight_id = 60731;

WITH occupied_seats_query AS (
    SELECT array_agg(seat_no) AS occupied_seats
    FROM boarding_passes
    WHERE flight_id = 60731
)
SELECT seat_no
FROM seats_comfort, occupied_seats_query
WHERE aircraft_code = '773' AND fare_conditions = 'Business'
  AND NOT (seats_comfort.seat_no = ANY(occupied_seats_query.occupied_seats));

WITH occupied_seats_query AS (
    SELECT COALESCE(array_agg(seat_no), '{}'::VARCHAR[]) AS occupied_seats
    FROM boarding_passes
    WHERE flight_id = 60622
)
SELECT seat_no
FROM seats_comfort, occupied_seats_query
WHERE aircraft_code = '773' AND fare_conditions = 'Economy'
  AND NOT (seats_comfort.seat_no = ANY(occupied_seats_query.occupied_seats));

-- WITH occupied_seats_query AS (SELECT count(ticket_no) AS occupied_seats
--                               FROM ticket_flights
--                               WHERE flight_id = 60622 AND fare_conditions = 'Economy'),
--      aircraft_code_query AS (SELECT aircraft_code FROM flights_v
--                              WHERE flight_id = 60622)
SELECT (        SELECT count(ticket_no) AS occupied_seats
                FROM ticket_flights
                WHERE flight_id = 60622 AND fare_conditions = 'Economy') as free_seats
FROM seats_comfort
WHERE seats_comfort.aircraft_code = (SELECT aircraft_code FROM flights_v
                                     WHERE flight_id = 60622)
  AND fare_conditions = 'Economy';

WITH occupied_seats_query AS (SELECT count(ticket_no) AS occupied_seats
                              FROM ticket_flights
                              WHERE flight_id = 60622 AND fare_conditions = 'Economy'),
     aircraft_code_query AS (SELECT aircraft_code FROM flights_v
                             WHERE flight_id = 60622)
SELECT count(seat_no) as free_seats
FROM seats_comfort, occupied_seats_query, aircraft_code_query
WHERE seats_comfort.aircraft_code = aircraft_code_query.aircraft_code
  AND fare_conditions = 'Economy';

SELECT *
FROM seats_comfort
WHERE seats_comfort.aircraft_code = '733'
  AND fare_conditions = 'Economy';