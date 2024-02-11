CREATE OR REPLACE VIEW mps AS
SELECT
    dp.day,
    dp.p1_production,
    dp.p2_production,
    dp.p3_production,
    dp.p4_production,
    dp.p5_production,
    dp.p6_production,
    dp.p7_production,
    dp.p8_production,
    dp.p9_production,

    w1.w1_p1_stock + w1.w1_p2_stock + w1.w1_p3_stock + w1.w1_p4_stock +
    w1.w1_p5_stock + w1.w1_p6_stock + w1.w1_p7_stock + w1.w1_p8_stock +
    w1.w1_p9_stock AS w1_stock,

    w2.w2_p1_stock + w2.w2_p2_stock + w2.w2_p3_stock + w2.w2_p4_stock +
    w2.w2_p5_stock + w2.w2_p6_stock + w2.w2_p7_stock + w2.w2_p8_stock +
    w2.w2_p9_stock AS w2_stock

FROM
    daily_production dp
JOIN
    w1_daily_stock w1 ON m.day = w1.day
JOIN
    w2_daily_stock w2 ON m.day = w2.day;
