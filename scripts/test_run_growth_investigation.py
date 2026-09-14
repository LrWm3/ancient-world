import unittest
from run_growth_investigation import command


class InvestigationPresetTests(unittest.TestCase):
    def test_baseline_does_not_enable_common_food(self):
        cmd = command(['--seed', '409', '--history-years', '50'])
        self.assertEqual(cmd[cmd.index('--farm-nutrient-retention') + 1], '0.95')
        self.assertEqual(cmd[cmd.index('--farm-phosphorus-release') + 1], '5e-07')
        self.assertNotIn('needs-based-food', cmd)
        self.assertEqual(cmd[-4:], ['--seed', '409', '--history-years', '50'])

    def test_explicit_control_arm_and_system_choices_are_preserved(self):
        cmd = command(['--headless', '--farm-nutrient-retention=0.85',
                       '--farm-phosphorus-release', '0.0000001',
                       '--enable-system', 'needs-based-food'])
        self.assertEqual(cmd.count('--headless'), 1)
        self.assertEqual(cmd.count('--farm-nutrient-retention'), 1)
        self.assertEqual(cmd[cmd.index('--farm-nutrient-retention') + 1], '0.85')
        self.assertEqual(cmd[cmd.index('--farm-phosphorus-release') + 1], '1e-07')
        self.assertEqual(cmd[-2:], ['--enable-system', 'needs-based-food'])


if __name__ == '__main__':
    unittest.main()
