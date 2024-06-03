DROP TABLE IF EXISTS seats_comfort;

CREATE TABLE seats_comfort
(
    aircraft_code   CHAR(3)         NOT NULL,
    seat_no         VARCHAR(4)      NOT NULL,
    fare_conditions VARCHAR(10)     NOT NULL,

    PRIMARY KEY (aircraft_code, seat_no)
);

INSERT INTO seats_comfort
SELECT * FROM seats;

UPDATE seats_comfort
SET fare_conditions = 'Comfort'
    FROM
(
    SELECT DISTINCT aircraft_code, seat_no
    FROM ticket_flights
             JOIN flights ON ticket_flights.flight_id = flights.flight_id
             JOIN boarding_passes ON ticket_flights.ticket_no = boarding_passes.ticket_no
        AND ticket_flights.flight_id = boarding_passes.flight_id
             JOIN prices ON flights.flight_no = prices.flight_no AND ticket_flights.amount = prices.amount
    WHERE prices.fare_conditions = 'Comfort'
) AS comfort_seats_query
WHERE seats_comfort.seat_no = comfort_seats_query.seat_no
  AND seats_comfort.aircraft_code = comfort_seats_query.aircraft_code
