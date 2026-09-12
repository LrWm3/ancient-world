import unittest
from compare_council_funding import arrears


class ActiveArrearsTests(unittest.TestCase):
    def sample(self, **extra):
        return dict(administrations=[dict(unpaid_months=900),
                                     dict(unpaid_months=3)], **extra)

    def test_inactive_counter_is_retained_but_excluded(self):
        self.assertEqual(arrears(self.sample(active_site_ids=[1])), (900, 3))

    def test_no_active_sites(self):
        self.assertEqual(arrears(self.sample(active_site_ids=[])), (900, 0))

    def test_legacy_is_unknown(self):
        self.assertEqual(arrears(self.sample()), (900, None))

    def test_invalid_counters_are_rejected_including_inactive_sites(self):
        for value in (-1, float('nan'), float('inf'), True, 1.5):
            sample = self.sample(active_site_ids=[1])
            sample['administrations'][0]['unpaid_months'] = value
            with self.subTest(value=value), self.assertRaises(ValueError):
                arrears(sample)

    def test_invalid_metadata_is_rejected(self):
        for ids in ([1, 1], [2], [-1], [True], [1.0], None):
            with self.subTest(ids=ids), self.assertRaises(ValueError):
                arrears(self.sample(active_site_ids=ids))


if __name__ == '__main__':
    unittest.main()
