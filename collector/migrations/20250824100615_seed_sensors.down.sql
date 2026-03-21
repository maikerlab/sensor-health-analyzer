DELETE
FROM sensors
WHERE device_id IN (
    '83e4eff3-25c1-4226-a940-959b46b9a8d0',
    '4bc2397a-c3d9-4ec4-a7d9-7e9da6e777a1',
    '91bcbf1f-811c-4f73-827c-9784dfaa73a9'
);

DELETE
FROM devices
WHERE id IN (
    '83e4eff3-25c1-4226-a940-959b46b9a8d0',
    '4bc2397a-c3d9-4ec4-a7d9-7e9da6e777a1',
    '91bcbf1f-811c-4f73-827c-9784dfaa73a9'
);