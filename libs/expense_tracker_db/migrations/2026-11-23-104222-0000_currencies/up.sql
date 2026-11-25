UPDATE currencies
SET name = 'Swiss Franc'
WHERE id = 1;

UPDATE currencies
SET symbol = '$', name = 'US Dollar'
WHERE id = 3;

INSERT INTO currencies (name, symbol)
VALUES
    ('Turkish Lira', '₺'),
    ('British Pound', '£'),
    ('Japanese Yen', '¥'),
    ('Indian Rupee', '₹'),
    ('Chinese Yuan', '¥'),
    ('Canadian Dollar', 'C$'),
    ('Australian Dollar', 'A$'),
    ('New Zealand Dollar', 'NZ$'),
    ('South African Rand', 'R'),
    ('Brazilian Real', 'R$'),
    ('Mexican Peso', '$'),
    ('Russian Ruble', '₽'),
    ('Singapore Dollar', 'S$'),
    ('Hong Kong Dollar', 'HK$'),
    ('Norwegian Krone', 'kr'),
    ('Swedish Krona', 'kr'),
    ('Danish Krone', 'kr'),
    ('Polish Zloty', 'zł'),
    ('Czech Koruna', 'Kč'),
    ('Hungarian Forint', 'Ft'),
    ('Israeli Shekel', '₪'),
    ('South Korean Won', '₩'),
    ('Thai Baht', '฿'),
    ('Indonesian Rupiah', 'Rp'),
    ('Malaysian Ringgit', 'RM'),
    ('Philippine Peso', '₱'),
    ('Vietnamese Dong', '₫'),
    ('Argentine Peso', '$'),
    ('Chilean Peso', '$'),
    ('Colombian Peso', '$'),
    ('Peruvian Sol', 'S/'),
    ('Uruguayan Peso', '$U'),
    ('Venezuelan Bolívar', 'Bs.'),
    ('Egyptian Pound', 'E£'),
    ('Moroccan Dirham', 'MAD'),
    ('Saudi Riyal', 'SR'),
    ('United Arab Emirates Dirham', 'AED'),
    ('Qatari Riyal', 'QR'),
    ('Kuwaiti Dinar', 'KD'),
    ('Omani Rial', 'OMR'),
    ('Bahraini Dinar', 'BD'),
    ('Jordanian Dinar', 'JD');