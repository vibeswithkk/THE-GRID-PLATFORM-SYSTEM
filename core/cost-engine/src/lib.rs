//! TGP Cost Calculation Engine
//!
//! Implements Formula 4.1 from TGP Blueprint:
//! C_total(J, t) = Σ [C_comp(j,t) + C_data(j,t) + C_idle(t)]

use serde::{Deserialize, Serialize};

/// Total cost breakdown for a job
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TotalCost {
    /// Compute cost (instance pricing)
    pub compute_usd: f64,
    /// Data transfer cost (egress/ingress)
    pub data_transfer_usd: f64,
    /// Idle resource opportunity cost
    pub idle_opportunity_usd: f64,
    /// Total cost
    pub total_usd: f64,
}

impl TotalCost {
    pub fn new(compute: f64, data_transfer: f64, idle: f64) -> Self {
        Self {
            compute_usd: compute,
            data_transfer_usd: data_transfer,
            idle_opportunity_usd: idle,
            total_usd: compute + data_transfer + idle,
        }
    }
}

/// Cost calculator implementing Formula 4.1 from TGP blueprint
#[derive(Debug, Clone)]
pub struct CostCalculator {
}

impl CostCalculator {
    pub fn new() -> Self {
        Self {}
    }

    /// Calculate C_comp: Compute cost
    /// 
    /// Formula: C_comp(j,t) = instance_price_per_hour * duration_hours * utilization_factor
    pub fn compute_cost(
        &self,
        instance_price_per_hour: f64,
        duration_hours: f64,
        utilization_factor: f64,
    ) -> f64 {
        instance_price_per_hour * duration_hours * utilization_factor
    }

    /// Calculate C_data: Data transfer cost
    /// 
    /// Formula: C_data(j,t) = data_size_gb * transfer_price_per_gb
    pub fn data_transfer_cost(&self, data_size_gb: f64, transfer_price_per_gb: f64) -> f64 {
        data_size_gb * transfer_price_per_gb
    }

    /// Calculate C_idle: Idle resource opportunity cost
    /// 
    /// This represents the cost of on-premise resources sitting idle
    pub fn idle_opportunity_cost(&self, idle_capacity_hours: f64, opportunity_cost_per_hour: f64) -> f64 {
        idle_capacity_hours * opportunity_cost_per_hour
    }

    /// Calculate total cost: C_total = C_comp + C_data + C_idle
    pub fn total_cost(
        &self,
        instance_price_per_hour: f64,
        duration_hours: f64,
        utilization_factor: f64,
        data_size_gb: f64,
        transfer_price_per_gb: f64,
        idle_capacity_hours: f64,
        opportunity_cost_per_hour: f64,
    ) -> TotalCost {
        let compute = self.compute_cost(instance_price_per_hour, duration_hours, utilization_factor);
        let data_transfer = self.data_transfer_cost(data_size_gb, transfer_price_per_gb);
        let idle = self.idle_opportunity_cost(idle_capacity_hours, opportunity_cost_per_hour);

        TotalCost::new(compute, data_transfer, idle)
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cost_calculation() {
        let calculator = CostCalculator::new();
        
        // $0.50/hour for 2 hours at 80% utilization
        let cost = calculator.compute_cost(0.5, 2.0, 0.8);
        assert_eq!(cost, 0.8); // $0.80
    }

    #[test]
    fn test_data_transfer_cost() {
        let calculator = CostCalculator::new();
        
        // 100GB at $0.09/GB
        let cost = calculator.data_transfer_cost(100.0, 0.09);
        assert_eq!(cost, 9.0); // $9.00
    }

    #[test]
    fn test_total_cost() {
        let calculator = CostCalculator::new();
        
        let total = calculator.total_cost(
            0.5,    // $0.50/hour instance
            2.0,    // 2 hours duration
            1.0,    // 100% utilization
            10.0,   // 10GB data transfer
            0.09,   // $0.09/GB transfer cost
            0.0,    // 0 idle hours
            0.0,    // $0 opportunity cost
        );

        assert_eq!(total.compute_usd, 1.0);
        assert!((total.data_transfer_usd - 0.9).abs() < 0.001); // FP precision
        assert_eq!(total.idle_opportunity_usd, 0.0);
        assert!((total.total_usd - 1.9).abs() < 0.001); // FP precision
    }
}
