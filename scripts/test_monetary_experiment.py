import copy
import unittest

from monetary_experiment import credit_funnel


def fixture():
    return {"credit": {
        "loans": [{"id": 0, "original_principal": 4.}],
        "rounds": [{
            "complete": True,
            "requests": [{"id": 9, "principal": 10.,
                          "terms": {"source": {"Export": {"contract": 1}}}}],
            "grants": [{"request": 9, "eligible": 10., "granted": 5.,
                        "decision": "Approved", "capacity": {
                            "lender_principal": 5., "lender_demand": 10.,
                            "borrower_principal": 5., "borrower_demand": 10.,
                            "source_receipts": 10., "source_demand": 10.,
                        }}],
            "loan_ids": [0],
        }],
    }}


class CreditReportingTests(unittest.TestCase):
    def test_request_grant_and_transfer_are_separate_and_tied_limits_count(self):
        report = credit_funnel(fixture())
        self.assertEqual(report["credit_requested_principal"], 10.)
        self.assertEqual(report["credit_eligible_principal"], 10.)
        self.assertEqual(report["credit_granted_principal"], 5.)
        self.assertEqual(report["credit_committed_round_principal"], 4.)
        self.assertEqual(report["credit_binding_capacity_counts"], {"borrower": 1, "lender": 1})
        self.assertEqual(report["credit_request_sources"], {"Export": 1})

    def test_missing_records_are_not_an_empty_observed_round(self):
        self.assertEqual(credit_funnel({"credit": {"loans": []}}),
                         {"credit_request_records_available": False})
        report = credit_funnel({"credit": {"loans": [], "rounds": []}})
        self.assertTrue(report["credit_request_records_available"])
        self.assertEqual(report["credit_recorded_requests"], 0)

    def test_old_grants_do_not_claim_capacity_was_unconstrained(self):
        h = fixture()
        del h["credit"]["rounds"][0]["grants"][0]["capacity"]
        report = credit_funnel(h)
        self.assertEqual(report["credit_eligible_requests_without_capacity_records"], 1)
        self.assertEqual(report["credit_binding_capacity_counts"], {})

    def test_incomplete_and_explicit_loans_are_visible(self):
        h = fixture()
        h["credit"]["rounds"][0]["complete"] = False
        h["credit"]["rounds"][0]["loan_ids"] = [None]
        report = credit_funnel(h)
        self.assertEqual(report["credit_incomplete_rounds"], 1)
        self.assertEqual(report["credit_committed_round_principal"], 0.)
        self.assertEqual(report["credit_loans_outside_rounds"], 1)

    def test_rejections_and_reordered_requests_match_by_id(self):
        h = fixture()
        r = h["credit"]["rounds"][0]
        r["requests"].insert(0, {"id": 7, "principal": 8.,
                                 "terms": {"source": {"AnnualTax": {"council": 0}}}})
        r["grants"].append({"request": 7, "eligible": 0., "granted": 0.,
                            "decision": "Risk", "capacity": None})
        r["loan_ids"].append(None)
        report = credit_funnel(h)
        self.assertEqual(report["credit_decisions"], {"Approved": 1, "Risk": 1})
        self.assertEqual(report["credit_requested_principal"], 18.)

    def test_invalid_receipts_fail_instead_of_looking_successful(self):
        for mutate in (
            lambda h: h["credit"]["rounds"][0]["loan_ids"].clear(),
            lambda h: h["credit"]["rounds"][0]["loan_ids"].__setitem__(0, 99),
            lambda h: h["credit"]["rounds"].append(copy.deepcopy(h["credit"]["rounds"][0])),
            lambda h: h["credit"]["rounds"][0]["grants"][0].update(granted=11.),
            lambda h: h["credit"]["rounds"][0]["grants"][0].update(eligible=float("nan")),
            lambda h: h["credit"]["rounds"][0]["grants"][0]["capacity"].update(source_demand=0.),
        ):
            h = fixture()
            mutate(h)
            with self.assertRaises(ValueError):
                credit_funnel(h)


if __name__ == "__main__":
    unittest.main()
