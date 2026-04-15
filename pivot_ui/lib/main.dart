import 'package:flutter/material.dart';
import 'package:pivot_ui/src/rust/frb_generated.dart'; // 🌉 The bridge
import 'package:pivot_ui/src/rust/api/simple.dart';
import 'dart:io'; // Gives us the File tool to copy data
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<void> main() async {
  // This turns on the bridge before the app starts drawing the screen
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Pivot Log',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const PivotHomePage(),
    );
  }
}

class PivotHomePage extends StatefulWidget {
  const PivotHomePage({super.key});

  @override
  State<PivotHomePage> createState() => _PivotHomePageState();
}

class _PivotHomePageState extends State<PivotHomePage> {
  final TextEditingController _endedController = TextEditingController();
  final TextEditingController _startedController = TextEditingController();

  // Add these state lists to hold the generated tags
  final List<String> _endedTasks = [];
  final List<String> _startedTasks = [];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Pivot Log ⏱️'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // --- ENDED TASKS TAG INPUT ---
            _buildTagInput(
              label: 'Ended Tasks (e.g. 吃早餐) - Press Enter',
              controller: _endedController,
              tags: _endedTasks,
              onTagAdded: (value) {
                setState(() {
                  _endedTasks.add(value);
                });
              },
              onTagDeleted: (tag) {
                setState(() {
                  _endedTasks.remove(tag);
                });
              },
            ),

            const SizedBox(height: 16),

            // --- STARTED TASKS TAG INPUT ---
            _buildTagInput(
              label: 'Started Tasks (e.g. rust) - Press Enter',
              controller: _startedController,
              tags: _startedTasks,
              onTagAdded: (value) {
                setState(() {
                  _startedTasks.add(value);
                });
              },
              onTagDeleted: (tag) {
                setState(() {
                  _startedTasks.remove(tag);
                });
              },
            ),

            const SizedBox(height: 24),

            ElevatedButton.icon(
              onPressed: () async {
                if (_endedController.text.trim().isNotEmpty) {
                  _endedTasks.add(_endedController.text.trim());
                }
                if (_startedController.text.trim().isNotEmpty) {
                  _startedTasks.add(_startedController.text.trim());
                }

                final endedList = List<String>.from(_endedTasks);
                final startedList = List<String>.from(_startedTasks);

                // 1. Flutter gets the saved JSON string (Web/Mobile safe!)
                final prefs = await SharedPreferences.getInstance();
                final savedJson = prefs.getString('pivot_database') ?? '';

                // 2. We initialize Rust from the string, not a file path!
                var db = await PivotDb.fromJson(jsonData: savedJson);

                // 3. Rust does the logic
                await db.logTransition(
                  endedTasks: endedList,
                  startedTasks: startedList,
                );

                // 4. Rust gives the updated JSON string back to Flutter
                final newJson = await db.toJson();

                // 5. Flutter saves it back to the browser/device storage!
                await prefs.setString('pivot_database', newJson);

                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Tick saved cross-platform! 💾🌐'),
                    ),
                  );
                }

                setState(() {
                  _endedTasks.clear();
                  _startedTasks.clear();
                  _endedController.clear();
                  _startedController.clear();
                });
              },
              icon: const Icon(Icons.add_task),
              label: const Text('Log Tick'),
              style: ElevatedButton.styleFrom(
                minimumSize: const Size(double.infinity, 50),
              ),
            ),
            const SizedBox(height: 16), // Adds space between the buttons
            OutlinedButton.icon(
              onPressed: () async {
                try {
                  // 1. Get the path to your internal database
                  final directory = await getApplicationDocumentsDirectory();
                  final filePath = '${directory.path}/pivot_data.json';
                  final currentFile = File(filePath);

                  if (!await currentFile.exists()) {
                    throw "Database file not found at $filePath";
                  }

                  // 2. Read the bytes
                  final bytes = await currentFile.readAsBytes();

                  // 3. Ask the OS for a save location
                  String? savePath = await FilePicker.saveFile(
                    dialogTitle: 'Save your database backup',
                    fileName: 'pivot_data_backup.json',
                    type: FileType.custom,
                    allowedExtensions: ['json'],
                    bytes:
                        bytes, // This saves the file automatically ON ANDROID
                  );

                  // 4. FIX FOR WINDOWS: If we have a path but no file was created
                  if (savePath != null) {
                    final exportFile = File(savePath);

                    // On Windows, 'saveFile' doesn't write the bytes, so we do it manually:
                    if (!Platform.isAndroid && !Platform.isIOS) {
                      await exportFile.writeAsBytes(bytes);
                    }

                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('Export successful! 📁')),
                      );
                    }
                  }
                } catch (e) {
                  if (context.mounted) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Export failed: $e ❌')),
                    );
                  }
                }
              },
              icon: const Icon(Icons.download),
              label: const Text('Export Data'),
              style: OutlinedButton.styleFrom(
                minimumSize: const Size(double.infinity, 50),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // Helper method to build the YouTube-style tag input
  Widget _buildTagInput({
    required String label,
    required TextEditingController controller,
    required List<String> tags,
    required Function(String) onTagAdded,
    required Function(String) onTagDeleted,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // The Wrap widget displays the chips and wraps to the next line automatically
        Wrap(
          spacing: 8.0, // horizontal gap between chips
          runSpacing: 4.0, // vertical gap between lines of chips
          children: tags.map((tag) {
            return Chip(
              label: Text(tag),
              deleteIcon: const Icon(Icons.cancel, size: 18),
              onDeleted: () =>
                  onTagDeleted(tag), // Removes the tag when the X is clicked
            );
          }).toList(),
        ),
        if (tags.isNotEmpty) const SizedBox(height: 8),
        TextField(
          controller: controller,
          decoration: InputDecoration(
            labelText: label,
            border: const OutlineInputBorder(),
          ),
          onSubmitted: (value) {
            // Triggered when the user presses Enter on the keyboard
            if (value.trim().isNotEmpty) {
              onTagAdded(value.trim());
              controller.clear();
            }
          },
        ),
      ],
    );
  }
}
