"""Defines a dummy test."""

import pytest


def test_dummy() -> None:
    assert True


@pytest.mark.slow
def test_slow() -> None:
    assert True
