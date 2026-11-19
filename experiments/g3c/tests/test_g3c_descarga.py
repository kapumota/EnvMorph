"""Regresiones sin red para las fuentes oficiales fijadas del PR #8."""
import io
import pathlib
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / 'docker'))
import preparar_debs as D


class ArchivoOficialTests(unittest.TestCase):
    def test_solo_las_seis_identidades_predeclaradas(self):
        self.assertEqual(len(D.OFFICIAL_POOL), 6)
        self.assertIn(('zip', '3.0-13ubuntu0.2'), D.OFFICIAL_POOL)
        self.assertTrue(D.official_url('findutils', '4.9.0-5build1').startswith(
            'https://archive.ubuntu.com/ubuntu/pool/main/f/findutils/'))
        self.assertIn('%2B', D.official_url('libimage-exiftool-perl', '12.76+dfsg-1'))
        with self.assertRaisesRegex(RuntimeError, 'ruta oficial fijada'):
            D.official_url('findutils', '4.10.0-3')

    def test_rechaza_bytes_sin_hash_fijado(self):
        class Respuesta(io.BytesIO):
            def geturl(self):
                return D.official_url('findutils', '4.9.0-5build1')
        class R:
            stdout = 'amd64\n'
        with tempfile.TemporaryDirectory() as t:
            with patch.object(D, 'urlopen', return_value=Respuesta(b'bytes no verificados')):
                with patch.object(D, 'run', return_value=R()):
                    with self.assertRaisesRegex(RuntimeError, 'SHA-256 incorrecto'):
                        D.descargar_archivo_oficial('findutils','4.9.0-5build1','0'*64,pathlib.Path(t))


if __name__ == '__main__':
    unittest.main()
