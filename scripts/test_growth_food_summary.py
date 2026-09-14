"""Independent boundary fixtures; no GPU or generated artifacts."""
import io
import json
import unittest
from unittest.mock import patch
from summarize_growth_food import summarize


def row(site, available, eaten):
    boundary = {"food_kg": 0, "ledger_kg": [0, 0, 0]}
    return dict(site=site, need_kg=100, eaten_kg=eaten,
        physically_available_kg=available, illness=0, cultivated_ha=1,
        land_capacity_ha=2, farm_workers=[0, 0, 0, 0],
        crop_probe=[[0, 0, 0, 0], [0, 0, 0, 1], [0, 0, 0, 0]],
        before_gpu=boundary, after_gpu=boundary, opening=boundary, closing=boundary)


class FoodSummaryTests(unittest.TestCase):
    def evaluate(self, rows):
        data = json.dumps(dict(month=1, food=rows, sites=[])) + "\n"
        with patch("builtins.open", return_value=io.StringIO(data)):
            return summarize("unused")[1]

    def test_transport_bound_is_distinct_from_local_access(self):
        r = self.evaluate([row(0, 40, 20), row(1, 150, 100)])
        self.assertEqual(r["physical_gap_kg"], 60)
        self.assertEqual(r["access_gap_kg"], 20)
        self.assertEqual(r["instant_redistribution_upper_bound_kg"], 50)
        self.assertEqual(r["world_physical_deficit_kg"], 10)

    def test_missing_boundary_is_not_silently_zero(self):
        bad = row(0, 100, 100)
        bad["opening"] = None
        with self.assertRaisesRegex(ValueError, "Incomplete"):
            self.evaluate([bad])


if __name__ == "__main__":
    unittest.main()
