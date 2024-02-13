CREATE OR REPLACE VIEW mps AS
SELECT
    dp.day,

    dp.p3, dp.p4, dp.p5, dp.p6, dp.p7, dp.p8, dp.p9,

    w1.p1 + w1.p2 + w1.p3 + w1.p4 + w1.p8 AS w1,
    w2.p3 + w2.p4 + w2.p5 + w2.p6 + w2.p7 + w2.p8 + w2.p9 AS w2
FROM
    daily_production dp
JOIN
    w1_daily_stock w1 ON dp.day = w1.day
JOIN
    w2_daily_stock w2 ON dp.day = w2.day;
