INSERT INTO devices(id, material_no, serial_no, custom_id, name, device_type, location)
VALUES ('83e4eff3-25c1-4226-a940-959b46b9a8d0', '123456', '12345678', NULL, 'Washing Machine', 'washing_machine',
        'Bathroom'),
       ('4bc2397a-c3d9-4ec4-a7d9-7e9da6e777a1', NULL, NULL, 'my-stove', 'Stove+Oven', 'stove', 'Kitchen'),
       ('91bcbf1f-811c-4f73-827c-9784dfaa73a9', NULL, NULL, 'my-napoleon', 'Grill', 'grill', 'Balcony');

-- Sensors of washing machine
INSERT INTO sensors (custom_id, device_id, channel, unit, description)
VALUES ('sensors/temp/1', '83e4eff3-25c1-4226-a940-959b46b9a8d0', 'water-temp', 'temperature',
        'current water temperature'),
       ('sensors/humid/1', '83e4eff3-25c1-4226-a940-959b46b9a8d0', 'drum', 'humidity', 'humidity inside drum'),
       ('sensors/speed/1', '83e4eff3-25c1-4226-a940-959b46b9a8d0', 'rotation-speed', 'speed', 'current rotating speed');

-- Sensors of stove+oven combo in the kitchen
INSERT INTO sensors (custom_id, device_id, channel, unit, description)
VALUES ('sensors/temp/2', '4bc2397a-c3d9-4ec4-a7d9-7e9da6e777a1', 'stove-1', 'temperature', 'front'),
       ('sensors/temp/3', '4bc2397a-c3d9-4ec4-a7d9-7e9da6e777a1', 'stove-2', 'temperature', 'rear'),
       ('sensors/humid/2', '4bc2397a-c3d9-4ec4-a7d9-7e9da6e777a1', 'hood', 'humidity', 'above stove');
