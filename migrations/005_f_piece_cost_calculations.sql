-- Function to calculate depreciation cost (Dc)
CREATE OR REPLACE FUNCTION calculate_depreciation_cost(
    raw_mat_arr_date_param int,
    raw_mat_cost_param money,
    dispatch_date_param int
) RETURNS numeric AS $$
DECLARE
    depreciation_cost numeric;
BEGIN
    -- Calculate depreciation cost using the provided formula
    depreciation_cost := raw_mat_cost_param * (dispatch_date_param - raw_mat_arr_date_param) * 0.01;

    RETURN depreciation_cost;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate total cost of a piece (Tc)
CREATE OR REPLACE FUNCTION calculate_total_cost(
    raw_mat_cost_param money,
    total_prod_time_param int,
    raw_mat_arr_date_param int,
    dispatch_date_param int
) RETURNS numeric AS $$
DECLARE
    production_cost numeric;
    depreciation_cost numeric;
    total_cost numeric;
BEGIN
    -- Calculate production cost using the provided formula
    production_cost := 1 * total_prod_time_param;

    -- Call the previous function to calculate depreciation cost
    depreciation_cost := calculate_depreciation_cost(raw_mat_arr_date_param, raw_mat_cost_param, dispatch_date_param);

    -- Calculate total cost using the provided formula
    total_cost := raw_mat_cost_param::numeric + production_cost + depreciation_cost;
    RETURN total_cost;
END;
$$ LANGUAGE plpgsql;
