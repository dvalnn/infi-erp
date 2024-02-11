-- Query to select piece_id, raw_mat_arr_date, raw_mat_cost, dispatch_date, total_prod_time,
-- depreciation cost (Dc), and total cost (Tc) for each piece from your table
CREATE OR REPLACE VIEW piece_costs AS
SELECT
    raw_mat_arr_date,
    raw_mat_cost,
    dispatch_date,
    total_prod_time,

    calculate_depreciation_cost(
      raw_mat_arr_date,
      raw_mat_cost,
      dispatch_date)
    AS depreciation_cost,

    calculate_total_cost(
      raw_mat_cost,
      total_prod_time,
      raw_mat_arr_date,
      dispatch_date)
    AS total_cost

FROM
    pieces;
