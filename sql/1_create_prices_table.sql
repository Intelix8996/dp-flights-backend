DROP TABLE IF EXISTS prices;

CREATE TABLE prices
(
    flight_no       CHAR(6)                 NOT NULL,
    fare_conditions VARCHAR(10)             NOT NULL,
    amount          INT                     NOT NULL,
    
    PRIMARY KEY (flight_no, fare_conditions)
);

INSERT INTO prices (flight_no, fare_conditions, amount)
SELECT DISTINCT flight_no, fare_conditions, min(DISTINCT amount) AS price
FROM ticket_flights
         JOIN flights ON ticket_flights.flight_id = flights.flight_id
WHERE fare_conditions != 'Comfort'
GROUP BY flight_no, fare_conditions;

INSERT INTO prices (flight_no, fare_conditions, amount)
SELECT DISTINCT flight_no, 'Comfort' as fare_conditions, max(DISTINCT amount) AS comfort_price
FROM ticket_flights
         JOIN flights ON ticket_flights.flight_id = flights.flight_id
GROUP BY flight_no, fare_conditions
HAVING fare_conditions = 'Economy' AND count(DISTINCT amount) = 2;
