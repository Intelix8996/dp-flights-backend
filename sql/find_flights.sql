WITH RECURSIVE flights_recur AS (
    SELECT
        ARRAY[f1.departure_airport]::VARCHAR[] AS path,
        f1.arrival_airport AS end_point,
        f1.scheduled_departure AS departure_time,
        f1.scheduled_arrival AS arrival_time,
        ARRAY[f1.flight_id]::INT[] as flight_id,
            0 as len
    FROM flights_v f1
    WHERE f1.departure_airport = ANY('{DME,SVO}'::VARCHAR[])
      AND f1.status = 'Scheduled'
      AND f1.scheduled_departure
        BETWEEN '2017-08-15 04:20:00.000000 +00:00'
        AND '2017-08-21 04:20:00.000000 +00:00'
      AND free_seats(occupied_seats(fn.flight_id, 'Economy'), aircraft_type(fn.flight_id), 'Economy') > 1

    UNION

    SELECT
        flights_recur.path || flights_recur.end_point,
        fn.arrival_airport,
        flights_recur.departure_time,
        fn.scheduled_arrival,
        flights_recur.flight_id || fn.flight_id,
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
    WHERE len < 1
)
SELECT
    flights_recur.path || flights_recur.end_point AS path,
    flights_recur.flight_id as flight_ids,
    departure_time,
    arrival_time,
    len AS connections
FROM flights_recur
WHERE flights_recur.end_point = ANY('{GDZ}'::VARCHAR[]);
